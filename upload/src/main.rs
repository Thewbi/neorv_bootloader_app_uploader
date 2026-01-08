use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::time::Duration;

use std::path::Path;

use iced::advanced::graphics::core::SmolStr;
use iced::alignment::Alignment::Center;
use iced::alignment::Alignment::Start;
use iced::widget::{Column, button, column, text, text_input};
use iced::window;
use iced::{
    Application, Element, Fill, Font, Function, Preset, Program, Subscription, Task, Theme,
};

pub fn main() -> iced::Result {
    application().run()
}

fn application() -> Application<impl Program<Message = Message, Theme = Theme>> {
    iced::application(Upload::new, Upload::update, Upload::view)
        .title(Upload::title)
        .window_size((1024.0, 512.0))
}

#[derive(Default)]
pub struct Upload {
    value: i64,
    file_to_upload: SmolStr,
    serial_port: SmolStr,
}

#[derive(Debug, Clone)]
enum Message {
    Connect,
    FileToUploadInputChanged(String),
    FileToUploadChanged(SmolStr),
    SerialPortInputChanged(String),
    SerialPortChanged(SmolStr),
}

fn subtle_test_style(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.extended_palette().background.strongest.color),
    }
}

impl Upload {
    fn title(&self) -> String {
        format!("NEORV32 Bootloader Uploader")
    }

    pub fn new() -> Self {
        Self {
            value: 0,

            file_to_upload: SmolStr::new(
                r"C:\Users\lapto\dev\VHDL\neorv32-setups\neorv32\sw\example\demo_blink_led\neorv32_exe.bin",
            ),
            //file_to_upload: SmolStr::new(r"C:\Users\lapto\dev\VHDL\neorv32-setups\neorv32\sw\example\hello_world\neorv32_exe.bin"),
            serial_port: SmolStr::new("COM5"),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::FileToUploadInputChanged(new_input) => {
                print!("{}\r\n", new_input);
                self.file_to_upload = SmolStr::new(new_input.as_str());
            }

            Message::FileToUploadChanged(new_file_to_upload) => {
                print!("{}\r\n", new_file_to_upload);
            }

            Message::SerialPortInputChanged(new_input) => {
                print!("{}\r\n", new_input);
                self.serial_port = SmolStr::new(new_input.as_str());
            }

            Message::SerialPortChanged(serial_port) => {
                print!("{}\r\n", serial_port);
            }

            Message::Connect => {
                let path = Path::new(self.file_to_upload.as_str());
                if !path.exists() {
                    print!("Path {} does not exist!\r\n", self.file_to_upload.as_str());
                    return;
                }

                // let mut port = serialport::new(self.serial_port.as_str(), 19_200)
                //     .timeout(Duration::from_millis(10000))
                //     .open()
                //     .expect("Failed to open port");

                let open_result = serialport::new(self.serial_port.as_str(), 19_200)
                    .timeout(Duration::from_millis(10000))
                    .open();

                let mut port = match open_result {
                    Ok(port) => port,
                    Err(error) => {
                        print!("Cannot open {}\r\n", self.serial_port.as_str());
                        return;
                    }
                };

                let mut string_buffer = String::new();

                let mut done: bool = false;

                while !done {
                    let mut serial_buf: Vec<u8> = vec![0; 100];
                    //let bytes_read = port.read(serial_buf.as_mut_slice()).expect("Found no data!");

                    let read_result = port.read(serial_buf.as_mut_slice());
                    let mut bytes_read = match read_result {
                        Ok(bytes_read) => bytes_read,
                        Err(error) => {
                            print!("Cannot read from {}\r\n", self.serial_port.as_str());
                            return;
                        }
                    };

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

                        let path = Path::new(self.file_to_upload.as_str());

                        if path.exists() {
                            let data: Vec<u8> = fs::read(path).unwrap();
                            port.write(&data[..]);
                            // wait for upload to be processed
                            std::thread::sleep(Duration::from_millis(1000));
                        } else {
                            print!("Path {} does not exist!", self.file_to_upload.as_str());
                        }
                    }

                    if string_buffer.contains("OK") {
                        string_buffer.clear();

                        // wait for upload to be processed
                        std::thread::sleep(Duration::from_millis(200));

                        // start executable
                        let output = "e".as_bytes();
                        port.write(output).expect("Write failed!");

                        // terminate this button click
                        done = true;
                    }
                }

                std::mem::drop(port);

                self.value += 3;
            }
        }
    }

    fn view(&self) -> Column<'_, Message> {
        column![
            button("Upload").on_press(Message::Connect),

            text_input("File:", &self.file_to_upload)
                    .id("file-to-upload")
                    .on_input(Message::FileToUploadInputChanged)
                    .on_submit(Message::FileToUploadChanged(SmolStr::new("")))
                    .padding(15)
                    .size(12)
                    .align_x(Start),

            text_input("Serial-Port:", &self.serial_port)
                    .id("serial-port")
                    .on_input(Message::SerialPortInputChanged)
                    .on_submit(Message::SerialPortChanged(SmolStr::new("")))
                    .padding(15)
                    .size(12)
                    .align_x(Start),

            text("1. Deploy the NEORV32 design to an FPGA board.")
                    .width(Fill)
                    .size(16)
                    .style(subtle_test_style)
                    .align_x(Start),

            text("2. Make sure the Bootloader is running. A LED should blink if the bootloader is accepting input.")
                    .width(Fill)
                    .size(16)
                    .style(subtle_test_style)
                    .align_x(Start),

            text("3. Press the Upload button in this application")
                    .width(Fill)
                    .size(16)
                    .style(subtle_test_style)
                    .align_x(Start),

            text("4. Press the reset button on the FPGA board.")
                    .width(Fill)
                    .size(16)
                    .style(subtle_test_style)
                    .align_x(Start),

            text("5. This uploader will wait 10 seconds for the Bootloader to respond after the reset. If the bootloader does not respond, this uploader application will silently fail and NOT UPLOAD your file!")
                    .width(Fill)
                    .size(16)
                    .style(subtle_test_style)
                    .align_x(Start),

        ]
        .padding(20)
        .align_x(Center)
    }
}
