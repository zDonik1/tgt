# logger.toml

## Default logger configuration

```toml
# `log_folder` is the folder where the log file will be created
log_folder = ".data"
# `log_file` is the name of the log file
log_file = "tgt.log"
# The rotation frequency of the log.
# The log rotation frequency can be one of the following:
# - minutely: A new log file in the format of log_folder/log_file.yyyy-MM-dd-HH-mm will be created minutely (once per minute)
# - hourly: A new log file in the format of log_folder/log_file.yyyy-MM-dd-HH will be created hourly
# - daily: A new log file in the format of log_folder/log_file.yyyy-MM-dd will be created daily
# - never: This will result in log file located at log_folder/log_file
rotation_frequency = "daily"
# The maximum number of old log files that will be stored
max_old_log_files = 7
# `log_level` is the level of logging.
# The levels are (based on `RUST_LOG`):
# - error: only log errors
# - warn: log errors and warnings
# - info: log errors, warnings and info
# - debug: log errors, warnings, info and debug
# - trace: log errors, warnings, info, debug and trace
# - off: turn off logging
log_level = "info"
```

## Example of a custom logger configuration

This is an example of a custom logger configuration. This configuration will be merged with the default configuration.
It means that the default logger configuration will be overwritten by the custom logger configuration.

```toml
log_level = "debug"
```