# twitch-chat-logger

This is a Twitch.tv bot that joins whatever channel you'd like and logs all messages it sees to a database, including deleted messages.

## Instances

| URL | Hosted By | Channels | Since (YYYY/MM/DD) |
| --- | --- | --- | --- |
| [chatlogs.absolucy.gay](https://chatlogs.absolucy.gay) | @Absolucy | [Jerma985](https://www.twitch.tv/jerma985), [nyanners](https://www.twitch.tv/nyanners), [Vinesauce](https://www.twitch.tv/Vinesauce), [Vargskelethor](https://twitch.tv/Vargskelethor) | 2022/09/07 |

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

## License

This software, including its source code, is subject to the terms of the [Mozilla Public License, v2.0](LICENSE.md).

### Amendment

I, @Absolucy, fully give permission for any of my code (including the entirety of this project, twitch-chat-logger), anywhere, no matter the license, to be used to train machine learning models intended to be used for general-purpose programming or code analysis.
