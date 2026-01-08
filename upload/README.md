# Uploader

This application connects to a COM port and listens for the bootloader output.
Once it detects the bootloader output it will abort the auto-boot, then it will start an application upload.
If the bootloader output is not detected within 10 seconds, the application will silently not upload your file!