# twitch-chat-logger

This is a Twitch.tv bot that joins whatever channel you'd like and logs all messages it sees to a database, including deleted messages.

## Configuration

The bot is configured via the `config.ron` file, in the working directory the binary is ran from. See [config.ron.example](config.ron.example) for an example.

`database`: A URL of the database to connect to. It will automatically apply migrations and such, the database just needs to exist.<br>
`port`: The port on which to host the search API on. If it's 0, the search API will not be hosted.

### `twitch`

`username`: The username of the bot.<br>
`access_token`: The access token of the bot. It should have the `chat:read` scope. You can get this (alongside `refresh_token`) via the Twitch CLI.<br>
`refresh_token`: The refresh token of the bot. It will automatically refresh the access token every so often. You can get this (alongside `access_token`) via the Twitch CLI.<br>
`client_id`: The client ID of the bot.<br>
`client_secret`: The client secret of the bot.<br>
`channels`: A list of Twitch channels to log. Case insensitive.<br>

## Wait, doesn't Twitch hate things like this?

Yup. Which is why, as a huge believer in freedom of information, I'm taking several precautions:

 - Every time I get it into a "stable" state (i.e I've just finished working on new features), I'll archive the entire thing on archive.org. This GitHub page should always be saved on archive.org
   - I also make sure to save the full code downloads, so you can always try to go to `https://github.com/Absolucy/twitch-chat-logger/archive/refs/heads/main.zip` or `https://github.com/Absolucy/twitch-chat-logger/archive/refs/heads/main.tar.gz` on archive.org!
 - I plan to make my own instance's database available via either IPFS and/or torrent in the future.


## License

This software, including its source code, is subject to the terms of the [Mozilla Public License, v2.0](LICENSE.md).
