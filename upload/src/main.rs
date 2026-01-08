use std::time::Duration;
use std::io::{self, Write, BufRead, BufReader};
use std::fs;

use iced::Center;
use iced::widget::{Column, button, column, text};

pub fn main() -> iced::Result {
    iced::run(Counter::update, Counter::view)
}

#[derive(Default)]
struct Counter {
    value: i64,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Connect,
    Increment,
    Decrement,
}

impl Counter {
    fn update(&mut self, message: Message) {
        match message {
            Message::Connect => {

                let mut port = serialport::new("COM5", 19_200)
                    .timeout(Duration::from_millis(10000))
                    .open()
                    .expect("Failed to open port");

                let mut string_buffer = String::new();

                while true {

                    let mut serial_buf: Vec<u8> = vec![0; 100];
                    let bytes_read = port.read(serial_buf.as_mut_slice()).expect("Found no data!");

                    let string_data = String::from_utf8(serial_buf).unwrap();
                    let substring = &string_data[..bytes_read];

                    println!("{:?}", substring);

                    string_buffer.push_str(substring);

                    if string_buffer.contains("Auto-boot") {

                        string_buffer.clear();

                        // wait for the auto-boot to abort
                        std::thread::sleep(Duration::from_millis(250));

                        // send any key to abort the Auto-boot
                        let output = "a".as_bytes();
                        port.write(output).expect("Write failed!");

                        // wait for the auto-boot to abort
                        std::thread::sleep(Duration::from_millis(250));

                        // select the upload command
                        let output = "u".as_bytes();
                        port.write(output).expect("Write failed!");

                    }

                    if string_buffer.contains("Awaiting neorv32_exe.bin") {

                        string_buffer.clear();

                        // wait for upload to be processed
                        std::thread::sleep(Duration::from_millis(200));

                        //let data: Vec<u8> = fs::read(r"C:\Users\lapto\dev\VHDL\neorv32-setups\neorv32\sw\example\demo_blink_led\neorv32_exe.bin").unwrap();
                        let data: Vec<u8> = fs::read(r"C:\Users\lapto\dev\VHDL\neorv32-setups\neorv32\sw\example\hello_world\neorv32_exe.bin").unwrap();
                        port.write(&data[..]);

                        // wait for upload to be processed
                        std::thread::sleep(Duration::from_millis(1000));

                    }

                    if string_buffer.contains("OK") {

                        string_buffer.clear();

                        // wait for upload to be processed
                        std::thread::sleep(Duration::from_millis(200));

                        // start executable
                        let output = "e".as_bytes();
                        port.write(output).expect("Write failed!");

                    }

                }

                std::mem::drop(port);

                self.value += 3;
            }
            Message::Increment => {
                self.value += 1;
            }
            Message::Decrement => {
                self.value -= 1;
            }
        }
    }

    fn view(&self) -> Column<'_, Message> {
        column![
            button("Connect").on_press(Message::Connect),
            button("Increment").on_press(Message::Increment),
            text(self.value).size(50),
            button("Decrement").on_press(Message::Decrement)
        ]
        .padding(20)
        .align_x(Center)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced_test::{Error, simulator};

    #[test]
    fn it_counts() -> Result<(), Error> {
        let mut counter = Counter { value: 0 };
        let mut ui = simulator(counter.view());

        let _ = ui.click("Increment")?;
        let _ = ui.click("Increment")?;
        let _ = ui.click("Decrement")?;

        for message in ui.into_messages() {
            counter.update(message);
        }

        assert_eq!(counter.value, 1);

        let mut ui = simulator(counter.view());
        assert!(ui.find("1").is_ok(), "Counter should display 1!");

        Ok(())
    }
}