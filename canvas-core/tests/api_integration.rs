use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use canvas_core::{get_course, list_courses, AuthConfig, CanvasConfig, DefaultsConfig};
use canvas_models::{CanvasHost, CanvasToken, CourseId};

struct MockResponse {
    status: u16,
    body: String,
    headers: Vec<(String, String)>,
}

fn start_mock_server<F>(build_responses: F) -> String
where
    F: FnOnce(String) -> Vec<MockResponse> + Send + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind listener");
    let addr = listener.local_addr().expect("local addr");
    let base_url = format!("http://{addr}");
    let responses = build_responses(base_url.clone());
    thread::spawn(move || {
        for response in responses {
            let (mut stream, _) = listener.accept().expect("accept");
            let _ = stream.set_read_timeout(Some(Duration::from_secs(1)));
            let mut buffer = Vec::new();
            let mut chunk = [0_u8; 1024];
            loop {
                match stream.read(&mut chunk) {
                    Ok(0) => break,
                    Ok(read) => {
                        buffer.extend_from_slice(&chunk[..read]);
                        if buffer.windows(4).any(|window| window == b"\r\n\r\n") {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            write_response(&mut stream, response);
        }
    });
    base_url
}

fn write_response(stream: &mut std::net::TcpStream, response: MockResponse) {
    let mut headers = response.headers;
    headers.push((
        "Content-Length".to_string(),
        response.body.as_bytes().len().to_string(),
    ));
    headers.push(("Content-Type".to_string(), "application/json".to_string()));

    let mut response_text = format!("HTTP/1.1 {} OK\r\n", response.status);
    for (name, value) in headers {
        response_text.push_str(&format!("{name}: {value}\r\n"));
    }
    response_text.push_str("\r\n");
    let _ = stream.write_all(response_text.as_bytes());
    let _ = stream.write_all(response.body.as_bytes());
}

#[test]
fn list_courses_paginates_with_mock_api() {
    let base_url = start_mock_server(|base_url| {
        let next_link = format!("{base_url}/api/v1/courses?page=2");
        vec![
            MockResponse {
                status: 200,
                body: r#"[{"id":1,"name":"Course 1"}]"#.to_string(),
                headers: vec![(
                    "Link".to_string(),
                    format!("<{next_link}>; rel=\"next\""),
                )],
            },
            MockResponse {
                status: 200,
                body: r#"[{"id":2,"name":"Course 2"}]"#.to_string(),
                headers: Vec::new(),
            },
        ]
    });
    let host: CanvasHost = base_url.parse().expect("host");
    let token: CanvasToken = "token123".parse().expect("token");
    let config = CanvasConfig {
        auth: AuthConfig { host, token },
        defaults: DefaultsConfig::default(),
    };

    let courses = list_courses(&config).expect("courses");
    assert_eq!(courses.len(), 2);
    assert_eq!(courses[0].id, 1);
    assert_eq!(courses[1].id, 2);
}

#[test]
fn get_course_reads_mock_response() {
    let base_url = start_mock_server(|_base_url| {
        vec![MockResponse {
            status: 200,
            body: r#"{ "id": 42, "name": "Biology 101", "course_code": "BIO-101" }"#
                .to_string(),
            headers: Vec::new(),
        }]
    });

    let host: CanvasHost = base_url.parse().expect("host");
    let token: CanvasToken = "token123".parse().expect("token");
    let config = CanvasConfig {
        auth: AuthConfig { host, token },
        defaults: DefaultsConfig::default(),
    };

    let course_id: CourseId = "42".parse().expect("course id");
    let course = get_course(&config, course_id).expect("course");
    assert_eq!(course.id, 42);
    assert_eq!(course.name.as_deref(), Some("Biology 101"));
    assert_eq!(course.code.as_deref(), Some("BIO-101"));
}
