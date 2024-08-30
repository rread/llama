# llama

Yet another cli for LLMs in Rust.

This is based on the OpenAI 1.0 API, primarily focused on the chat completion parts. The main purpose is to learn more
about LLMs and Rust at the same time. I'm not terribly proficient in either.

At this point this is simple REPL for interacting with GPT. The next step is to add support for /commands so user can
tweak parameters or reset the conversation or whatever.

You can either use the environment variable OPENAI_API_KEY or put your key in a config file like this:

```ini
[openai]
api_key=YOUR_API_KEY_HERE
```

The default location for the config is `~/.config/openai.ini`

