# Startup Configuration

Notedeck can be configured to automatically connect to a relay and load your account on startup using a configuration file.

## Configuration File Location

Create a file named `startup_config.json` in:
- **Linux**: `~/.local/share/notedeck/settings/startup_config.json`
- **macOS**: `~/Library/Application Support/notedeck/settings/startup_config.json`
- **Windows**: `%APPDATA%\notedeck\settings\startup_config.json`

## Configuration Format

The configuration file is a JSON file with the following format:

```json
{
  "relay": "wss://relay.damus.io",
  "nsec": "nsec1your_private_key_here"
}
```

### Fields

- **`relay`** (optional): The WebSocket URL of the relay you want to connect to
  - Example: `"wss://relay.damus.io"`
  - If not specified, the application will use default relays

- **`nsec`** (optional): Your Nostr private key in nsec format
  - Example: `"nsec1..."`
  - This will be used to automatically create your account on startup
  - **Keep this file secure!** Your nsec is your private key and should never be shared

## Example Configuration

See `startup_config.json.example` in the root directory for a template.

## Security Notes

- **Never share your nsec** with anyone
- **Keep your startup_config.json file secure** with appropriate file permissions
- **Back up your nsec** in a safe location
- On Linux/macOS, you can set secure permissions with:
  ```bash
  chmod 600 ~/.local/share/notedeck/settings/startup_config.json
  ```

## Notes

- Both fields are optional - you can specify just the relay, just the nsec, or both
- If the configuration file doesn't exist, the application will start with default settings
- The startup configuration is loaded once during application startup
- Changes to the file require restarting the application to take effect
