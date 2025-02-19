Twitch Chat Logger is an app focused on connecting to Twitch's IRC servers,
parsing the incoming messages, and logging them to a mysql database.

Contained is an IRC client that will connect to a list of channels given in the
app's config under `config/config.yml`. Each message from these channels will be
parsed into the tables defined under the `migration` and `entities`. These
include things like Twitch emotes, messages, subscriptions, streams, users etc.

# Setup

Setting up the app will require you to set a few values under the config.
Required fields will be indicated by a \* in a comment above the example.

Any values split by `|` is a list of the possible options.

The available configuration values are as follows:

```
# Default is error
logLevel: error|warn|info|debug|trace

# Default will spit out chat logs in the path the program is run.
loggingDir: "path/to/logging/dir"

# Prefixes any log files with the string given. Default is nothing
loggingFilenamePrefix: ""

# Determines how often a new log file will be created.
# Default is Daily
loggingRollAppender: minute|hour|day|never

# * This is required.
# The list of channels to connect to and track.
channels: ["channel_1_name", "channel_2_name"]

# Sets a limit for how often queries can be made to Twitch
# Default is calculated based on the amount of channels in the list.
queriesPerMinute: 0

# * This is required.
# Your Twitch account name.
twitchNickname: "your_name_here"

# * This is required.
# Your Twitch access token obtained from `https://twitchtokengenerator.com/`
accessToken: "Your_Token_Here"

# * This is required.
# Your Twitch client ID obtained from `https://twitchtokengenerator.com/`
clientId: "Your_Token_Here"

# * If there is no $USER, then this is required.
# The username used to sign into MySql. Defaults to $USER if doesn't exist.
databaseUsername: "YourNameHere"

# The address used to connect to the MySql database.
# Defaults to `localhost:3306`.
databaseHostAddress: ""

# The name of the database.
# Defaults to `twitch_tracker_db`.
database = ""

# The password used to connect to mysql.
# Uses the `DATABASE_PASSWORD` environment variable. Otherwise defaults to `password`.
sqlUserPassword: ""

# The API key obtained from `https://pastebin.com/doc_api`.
# Used for automatically uploading reports with the `database_report_generator` to Pastebin.
# The `-f` feature flag can be used to output reports to file instead.
pastebinApiKey: "Your_Key_Here"
```

Once all the required fields are set, you can run the base program which will
connect to Twitch. Then start parsing messages and logging them to the database.
