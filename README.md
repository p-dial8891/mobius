# Mobius
A password manager that uses the raspberry pi as a USB dongle. The raspberry pi can be turned into a USB dongle with the Raspberry Pi OS and the scripts found in [Symbiote](https://github.com/p-dial8891/symbiote).

## Design
The software is composed of two components - server and client. The server runs on the rasperry pi and looks up requested data from local storage. The client runs on the host computer, currently only a windows PC and sends commands to the server. On receiving a command, the server emulates a USB HID keyboard and outputs the key presses required to spell out the data, ie the username and password. The client listens for key presses on the host computer - specifically F8 for username and F10 for password - and issues commands to get the credential. The credential id has to be specified at the command line when starting the client program. The client and server program need to aware of the IP address and port number when issuing the command on the command line.

The user credentials are stored on the raspberry pi under the name "input.json" and is not encrypted. In upcoming updates, it is planned to store the file on an encrypted partition.

## Usage
Server:  
`sudo server --secret MasterPassword --port 50051`

Client:  
`client.exe --server-addr 169.254.24.24:50051 --secret MasterPassword --id Website99`

The server-addr is optional and will default to the above if not specified. The secret and id above are examples and should correspond to the user's application.

