use chess_uci::Request;

fn main() {
    let mut buffer = String::new();
    loop {
        buffer.clear();
        let amount = std::io::stdin().read_line(&mut buffer).unwrap();
        if amount == 0 {
            return;
        }
        match buffer.trim().parse::<Request>() {
            Ok(x) => {
                println!("{x:?}")
            }
            Err(e) => {
                println!("{e}")
            }
        }
    }
}
