# yaib: Yet Another i3 Bar

yaib is a very simple (at this time of writing) status bar for the i3 window
manager (X11) that leverages async computation to limit performance
bottlenecks. This results in a very resource efficient bar capable of
displaying statistics and other things you'd like to display.

yaib is very immature at this writing. Lots of things you want won't be here.

Some features:

-   Expandable. Each block has an `icon` value which can then be clicked on to
    expand it. Combine it with the urgency coloring values, and you don't have to
    see text updating all day; just the colors when it matters.
-   Pages: Flip between lots of different pages using the arrows. This way you
    can hide less important things you care about in your bar, but get to them
    when you want to.
-   Resource light: yaib is very small and uses almost no resources.

For an example of the expansion, here's the expanded Disk section in the
default configuration:

<img style="height: 25px; width: auto" src="one.png" />

And here it is collapsed (just click):

<img style="height: 25px; width: auto" src="two.png" />

## Installation

Release:

```
cargo install yaib
```

Development (recommended):

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

There is an [example](example_config.yaml) configuration file. This
configuration file can either be specified by setting `YAIB_CONFIG` in the
environment, or by making a file in `$XDG_CONFIG_HOME/yaib/yaib.config.yaml`.

Field descriptions follow:

-   `update_interval` is the amount of time to wait before polling the system,
    and displaying new stats. It is specified in [fancy duration
    format](https://docs.rs/fancy-duration/latest/fancy_duration/struct.FancyDuration.html)
    which you can read more about at that link.
-   `pages` is a list of pages to flip through. Each page consists of a list of items:
    -   `name` is the name of the block. It is required, and must be unique for all blocks.
    -   `icon` is the short initial clickable content. Not supported on static
        values. If not provided, it will display the formatted content always.
    -   `urgency` is a 3-element tuple of values that are all under 100. They
        correspond to urgency values, green/yellow/red. Not supported on static
        or music values. When under the minimum, the default text color is
        used.
    -   `urgency_colors` is a 3-element tuple of `#rrggbb` values. These values
        are used when the urgency thresholds are set.
    -   `type` is the type of block. `value` and `format` are dependent on this
        type, so they will be specified with the type below:
        -   `command` runs a command. It does not run it through a shell, and
            tokenizes the value by whitespace. The value is the command to run.
            `update_interval` can be used to override the global
            `update_interval` for slow running or needlessly updating commands.
            See `example_command.sh` for more information. The command must
            emit (and only emit) a JSON blob with the following three
            parameters:
            -   `name`: this is the name of the block you configured it with, so it can map back.
            -   `value`: this is the data you want to show in the bar. The icon
                will be automatically concatenated if it exists.
            -   `percent`: this is optional, an integer from 0-100 which helps
                with urgency coloring.
        -   `dynamic` is only for types which are updated by the unix socket
            (see below). It carries no value and communicates no urgency and
            has no format.
        -   `static` just displays a static string set in the `value`. No
            formatting is applied.
        -   `music` displays several options for listing the current music track
            playing via MPRIS (e.g., spotify, xmms). No value is used.
            -   `%artist` is the current artist
            -   `%title` is the current track title
            -   `%pct_played` is the whole number percentage of how far along in the track you are.
            -   `%total_played` is the `minute:second` time well suited for regular updates.
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

## Unix Socket

**NOTE:** This layer is likely to be changed dramatically in the future. It is
recommended that if you use this feature, you use it through `yaib` commands,
and not writing to the socket directly, as the protocol is certain to change.

You can write blocks using the JSON format also used for the `command` type.
One block per write; use `yaib write-block '<block json>'` to write directly to
the socket. The socket is also located at `/tmp/yaib.sock`; only the most
recent copy of `yaib` running will respond to it, but you can use this with
`nc` et al to control it. Just barf some JSON at the socket. See
[example_command.sh](example_command.sh) for an example of the output format.

Whatever the block's `name` value is set to will replace the block in the bar.
If this block is not of a `dynamic` type in the configuration, it will not
persist and be overwritten by new collection data in the next iteration (this
behavior is expected to change in the future).

## License

MIT

## Author

Erik Hollensbe <git@hollensbe.org>
