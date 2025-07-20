Twitch Chat Logger is an app focused on connecting to Twitch's IRC servers,
parsing the incoming messages, and logging them to a mysql database.

Contained is an IRC client that will connect to a list of channels given in the
app's config under `config/config.yml`. Each message from these channels will be
parsed into the tables defined under the `migration` and `entities`. These
include things like Twitch emotes, messages, subscriptions, streams, users etc.
