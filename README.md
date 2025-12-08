# Mac Display Configuration

Some resolutions and capabilities are hidden by Mac OS.  Not sure why, but this can help you find and set them.

### Usage

List all displays and their available modes:
```shell
displayconfig list
```

Get the current brightness percentage for all displays:
```shell
displayconfig get-brightness
```

Get brightness for a specific display:
```shell
displayconfig get-brightness --display 1
```

Set brightness for a specific display (0-100%):
```shell
displayconfig set-brightness --display 1 --brightness 50
```

Set display mode:
```shell
displayconfig set-mode --display 798186BE-D89C-4988-871A-E111BFFBEA68 --mode 1
```

## Resources
```
https://github.com/w0lfschild/macOS_headers
https://github.com/alin23/mac-utils
```