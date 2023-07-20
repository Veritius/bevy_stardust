# Stardust authentication server
The authentication server acts as a secure communication bridge between the game server and client.
The only purpose of the authentication server is to allow a secure key exchange, at which point the game server and client disconnect and use the normal Stardust UDP protocol.

## Usage
At minimum, to run a Stardust auth server, you need:
- A domain (like google.com, can be any top level domain)
- A certificate for your domain (you can get one [here](https://letsencrypt.org/), but there are other CAs)
- The private key for the certificate (you get this when making a certificate, don't lose it!)
- Somewhere to host the server

You must then set your domain name to be the authentication server of choice for your game.
This is documented in the `bevy_stardust_shared` crate.

**You should run your own auth server for your game. The authentication server can see the keys in the key exchange, and a bad actor can manipulate them.**

### Config file
Your average configuration file will look like this.
```
[server]
port = 25807
protocol_id = 123456789

[encryption]
certificate = "certificate.pem"
privatekey = "privatekey.pem"

[logging]
verbose = false

[safety]
disconnect_time = 30
```

All the configuration options are as follows.
- **server/port** - A number from ranging `1024` to `65535` inclusive, defaulting to `25807`. See [here](https://en.wikipedia.org/wiki/Port_(computer_networking)) for details.
- **server/protocol_id** - The unique protocol id of your game. See the shared crate for more information.
- **encryption/certificate** - The path to/name of your certificate file.
- **encryption/privatekey** - The path to/name of your private key file.
- **logging/verbose** - Enables verbose logging.
- **safety/disconnect_time** - Disconnects clients after a certain amount of time, in seconds.