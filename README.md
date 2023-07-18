<h2 align="center">CoEmu (COServer Emulator)</h2>
<div align="center">
  <strong>
        Fully Featured Conquer Online 5017 Server Emulator
  </strong>
</div>
<div align="center">
  <sub>
      ⚠ Still Under Constructions ⚠.
  </sub>
</div>

## About

CoEmu is a Conquer Online server project containing an account server and game server. The account server authenticates players, while the game server services players in the game world. This simple two-server architecture acts as a good introduction into server programming and networking. The server is interoperable with the Conquer Online game client, patch 5017 (not provided by this project).

## Build and Run

1. [Install Rust](https://rustup.rs/)

2. Use Nightly Rust (it offers speed compile-time).

```bash
rustup default nightly
```

3. Clone and Configure

```bash
git clone https://github.com/shekohex/coemu && cd coemu && cp .env.example .env # edit the env file if you want.
```

4. Start Database

We are using [Sqlite](https://sqlite.org/) as Database for storing all of server data and states.

To make things easy, Install [sqlx-cli](https://github.com/launchbadge/sqlx/tree/master/sqlx-cli) for running Database Migrations.

```bash
$ cargo install sqlx-cli --no-default-features --features rustls,sqlite
```

that will take some time, then run

```bash
sqlx migrate run --database-url 'sqlite://data/coemu.db?mode=rwc'
```

Good, let's build the servers!

5. Build and Run Server

Now After the database is ready, run the servers

- **Auth Server**

```bash
$ cargo auth
```

- **Game Server**

```bash
$ cargo game
```

## Game Client Download

> Note: This Game is only available for Microsoft Windows OS.
> So if you willing to use it on Mac or Linux, I highly recommend looking at https://www.playonlinux.com/en/ or use a Windows VM.

This Server Emulator is build for Conquer Online version 5017. you can download this version from [here](https://mega.nz/folder/xJQnlSRD#d4oLOGq2LNbKm_k8kYVKJw/file/dMx3jDoC).

## Documentations

Currently We try to Write Every Single Detail about anything in the codebase it self, so at the end we have a self-documanted codebase.
all you have to do to view docs, Run:

```
$ cargo doc --no-deps --document-private-items --open
```

## FAQ

1. How do I configure the client?

- You may connect to your server instance without modifying client code. Create a shortcut to Conquer.exe and add "blacknull" as a target command-line argument. Open server.dat and enter the following (adding your IP address). Be sure not to specify an internal IP address (must be external or a loopback adapter). If you specify an internal IP address, the client will throw an error: "Server.dat is damaged".

```ini
[Header]
GroupAmount=1
Group1=GroupPic4

[Group1]
ServerAmount=1

Server1=CoEmu
Ip1=
Port1=9958
ServerName1=CoEmu
HintWord1=
Pic1=servericon33

```

2. How do I create an account?

To Create an Account, you should open the Database on a Database Manager (something like [Datagrip](https://www.jetbrains.com/datagrip/) or [alternatives](https://www.slant.co/options/210/alternatives/~datagrip-alternatives)) then in the `accounts` table you should create a new raw with your account information, you should only need to input the `username` and `password`.
Please note that the `password` needs to be hashed using [Bcrypt](https://en.wikipedia.org/wiki/Bcrypt) .. You could use the provided tool to get a hashed password, just run

```bash
$ cargo hash-pwd <password>
```

for example

```
$ cargo hash-pwd test
$2b$12$iSrnkacd/i/8eZr5pBoDlO5qcbLmLUWGQ6IN.oQuemnlRKU/NExIW
```

After creating the account, you should be able to login and create your character :)

## Resources

- [Epvp](https://elitepvpers.com/forum/co2-private-server)
- [ConquerWiki](https://www.conquerwiki.com/doku.php?id=start)
- [Comet](https://gitlab.com/spirited/comet)

## Contributing

Want to join us? Check out our ["Contributing" guide][contributing] and take a
look at some of these issues:

- [Issues labeled "good first issue"][good-first-issue]
- [Issues labeled "help wanted"][help-wanted]

[contributing]: .github/CONTRIBUTING.md
[good-first-issue]: https://github.com/shekohex/coemu/labels/good%20first%20issue
[help-wanted]: https://github.com/shekohex/coemu/labels/help%20wanted

## Legality

<sub>
Algorithms and packet structuring used by this project for interoperability with the Conquer Online game client is a result of reverse engineering. By Sec. 103(f) of the DMCA (17 U.S.C. § 1201 (f)), legal possession of the Conquer Online client is permitted for this purpose, including circumvention of client protection necessary for archiving interoperability. CoEmu is a non-profit, academic project and not associated with TQ Digital Entertainment. All rights over CoEmu are reserved by Shady Khalifa "shekohex". All rights over the game client are reserved by TQ Digital Entertainment.
</sub>

## License

<sup>
Licensed under <a href="LICENSE">GPL v3.0 license</a>.
</sup>

<br/>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the GPL-3.0 license, shall
be licensed as above, without any additional terms or conditions.
</sub>
