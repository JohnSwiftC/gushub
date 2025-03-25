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
    Use 'clients' to show connected clients\n\
    Use 'mainmenu' to exit an interactive shell\n\
    Use 'close [id]' to remove a shell from the clients list\n\
    Input a shell number to interact\n\
    ");

    loop {

        print!("GusHub >");
        stdout().flush().unwrap();
        let mut command = String::new();
        stdin().read_line(&mut command).expect("stdin failure");

        // We now clear out the buffer from the accepting thread
        {
            let mut cb = client_buffer.lock().unwrap();
            while let Some(client) = cb.pop() {
                clients.push(client);
            }
        }

        // Show clients if it was requested
        if command.trim() == "clients" {
            if clients.len() == 0 {
                println!("No connected clients!");
            }

            for (i, client) in clients.iter().enumerate() {
                println!("({}) - ({})", i + 1, client.address());
            }

            continue;
        }

        // Close shell command
        // Please clean this up later, it looks horrible
        let mut command_iter = command.trim().split_whitespace();
        match command_iter.next() {
            Some(s) => {
                // Now check for command
                if s == "close" {
                    if let Some(o) = command_iter.next() {
                        if let Ok(n) = o.parse::<usize>() {
                            // of course vec remove has to panic on a bad index, hold on
                            if n >= 1 && n <= clients.len() {
                                let _ = clients.remove(n - 1); // Throw away the result, we dont need it.
                            }
                        }
                    }
                }
            },
            None => ()
        }

        // If the command was a number, drop 
        let choice = match command.trim().parse::<usize>() {
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
            let mut buf = vec![0; 1024];
            let n = stdin().read(&mut buf).unwrap();
            if String::from_utf8_lossy(&buf[..n]).trim() == "mainmenu" {break;} // Figure this out later
            writer.write_all(&buf[..n]).unwrap();
        }
    });

    thread::spawn(move || {
        loop {
            if let Ok(_) = receiver.try_recv() {break;}

            let mut buf = vec![0; 1024];
            match reader.read(&mut buf) {
                Ok(0) => break,
                Err(_) => break,
                Ok(n) => {
                    stdout().write_all(&buf[..n]).unwrap();
                    stdout().flush().unwrap();
                },
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