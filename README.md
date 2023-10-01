# yaib: Yet Another i3 Bar

yaib is a very simple (at this time of writing) status bar for the i3 window
manager (X11) that leverages async computation to limit performance
bottlenecks. This results in a very resource efficient bar capable of
displaying statistics and other things you'd like to display.

yaib is very immature at this writing. Lots of things you want won't be here.

One feature that is someone novel about yaib is that it is expandable. Each
block has an `icon` value which can then be clicked on to expand it. Combine it
with the urgency coloring values, and you don't have to see text updating all
day; just the colors when it matters.

Here's the expanded Disk section in the default config:

<img style="height: 25px; width: auto" src="one.png" />

And here it is collapsed (just click):

<img style="height: 25px; width: auto" src="two.png" />

## Installation

```
cargo install --git https://github.com/erikh/yaib
```

## Execution & Setup

```
yaib
```

Will emit the bar's contents to standard output in JSON format.

To integrate it into your i3 installation, provide a stanza like so in your
`~/.config/i3/config` file; remember to remove any other block like it.

**NOTE:** in this block, you must replace `$HOME` with your home directory.

```
bar {
    font pango:monospace 10, FontAwesome 10
    position bottom
    status_command $HOME/.cargo/bin/yaib
    colors {
        separator #666666
        background #222222
        statusline #dddddd
        focused_workspace #0088CC #0088CC #ffffff
        active_workspace #333333 #333333 #ffffff
        inactive_workspace #333333 #333333 #888888
        urgent_workspace #2f343a #900000 #ffffff
    }
}
```

Do this and reload your configuration (mod + shift + r by default) and the bar
should appear!

## Configuration

There is an [example](example.yaml) configuration file. This configuration file
can either be specified by setting `YAIB_CONFIG` in the environment, or by
making a file in `$XDG_CONFIG_HOME/yaib/yaib.config.yaml`.

The configuration file looks like this:

```yaml
update_interval: "1s"
pages:
    - - name: static
        type: static
        value: "yaib rules"
      - name: cpu
        type: cpu
        format: "CPU: %usage%"
      - name: disk
        type: disk
        value: "/"
        format: "Disk: T: %total, U: %usage"
      - name: memory
        type: memory
        format: "Mem: T: %total, U: %usage"
      - name: load
        type: load
        format: "1m Load: %1"
      - name: time
        type: time
        format: "%a %m/%d %I:%M%P"
```

Field descriptions follow:

-   `update_interval` is the amount of time to wait before polling the system,
    and displaying new stats. It is specified in [fancy duration
    format](https://docs.rs/fancy-duration/latest/fancy_duration/struct.FancyDuration.html)
    which you can read more about at that link.
-   `pages` is a list of pages to flip through, and is unsupported and planned
    in the future. Merely fill out the first page via a YAML list of block
    items with the following properties:
    -   `name` is the name of the block. It is required, and must be unique for all blocks.
    -   `icon` is the short initial clickable content. Not supported on static
        values. If not provided, it will display the formatted content always.
    -   `urgency` is a 3-element tuple of values that are all under 100. They
        correspond to urgency values, green/yellow/red. Not supported on static
        values. When under the minimum, the default text color is used.
    -   `urgency_colors` is a 3-element tuple of `#rrggbb` values. These values
        are used when the urgency thresholds are set.
    -   `type` is the type of block. `value` and `format` are dependent on this
        type, so they will be specified with the type below:
        -   `static` just displays a static string set in the `value`. No
            formatting is applied.
        -   `cpu` are CPU metrics. Both `%count` (number of CPUs) and `%usage`
            are available as format strings.
        -   `disk` are storage metrics. The `value` is a mount point.
            -   `%total` is the total user storage
            -   `%usage` is the amount used
            -   `%pct` is the percent of disk used.
        -   `memory` are memory metrics. No value is used.
            -   `%total` is the total user memory
            -   `%usage` is the amount used
            -   `%swap_total` is the amount of swap available
            -   `%swap_usage` is the amount of swap used
            -   `%pct` is the percent of memory used.
            -   `%pct_swap` is the percent of swap used.
        -   `load` are memory metrics. No value is used.
            -   `%1` is the one minute load average
            -   `%5` is the five minute load average
            -   `%15` is the fifteen minute load average
        -   `time` are time metrics. No value is used. The format is [chrono's
            strftime
            format](https://docs.rs/chrono/latest/chrono/format/strftime/index.html)

## License

MIT

## Author

Erik Hollensbe <git@hollensbe.org>
