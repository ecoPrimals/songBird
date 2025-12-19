// Simple Toadstool Demo Server for Albatross
use std::net::TcpListener;
use std::io::{Read, Write};
use std::thread;

fn main() {
    println!("🍄 Toadstool Demo Server Starting...");
    let listener = TcpListener::bind("0.0.0.0:7878")
        .expect("Failed to bind to port 7878");
    println!("✅ Listening on 0.0.0.0:7878");
    println!("   Ready for Albatross benchmarks!");
    
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                thread::spawn(move || {
                    let mut buffer = [0; 1024];
                    if let Ok(_) = stream.read(&mut buffer) {
                        let request = String::from_utf8_lossy(&buffer);
                        
                        let response = if request.contains("GET /health") {
                            "HTTP/1.1 200 OK\r\n\
                             Content-Type: application/json\r\n\
                             Access-Control-Allow-Origin: *\r\n\
                             \r\n\
                             {\"status\":\"ok\",\"service\":\"toadstool\",\"version\":\"0.1.0\",\"ready_for\":\"albatross\"}"
                        } else if request.contains("POST /compute") {
                            // Simulate compute workload
                            thread::sleep(std::time::Duration::from_millis(10));
                            "HTTP/1.1 200 OK\r\n\
                             Content-Type: application/json\r\n\
                             \r\n\
                             {\"status\":\"completed\",\"result\":\"computed\",\"latency_ms\":10}"
                        } else {
                            "HTTP/1.1 200 OK\r\n\
                             Content-Type: text/plain\r\n\
                             \r\n\
                             Toadstool Demo Server - Ready for Albatross!"
                        };
                        
                        stream.write_all(response.as_bytes()).ok();
                    }
                });
            }
            Err(e) => eprintln!("Connection error: {}", e),
        }
    }
}
