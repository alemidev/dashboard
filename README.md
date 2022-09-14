# dashboard
![screenshot](https://data.alemi.dev/dashboard.png)
A data aggregating dashboard, capable of periodically fetching, parsing, archiving and plotting data.

### Name
Do you have a good name idea for this project? [Let me know](https://alemi.dev/suggestions/What%27s%20a%20good%20name%20for%20the%20project%3F)!

## How it works
This software periodically (customizable interval) makes a GET request to given URL, then applies all metric JQL queries to the JSON output, then inserts all extracted points into its underlying SQLite.
Each panel displays all points gathered respecting limits, without redrawing until user interacts with UI or data changes.
If no "x" query is specified, current time will be used (as timestamp) for each sample "x" coordinate, making this software especially useful for timeseries.

## Usage
This program will work on a database stored in `$HOME/.local/share/dashboard.db`. By default, nothing will be shown.
To add sources or panels, toggle edit mode (top left). Once in edit mode you can:
* Add panels (top bar)
* Add sources (in source sidebar, bottom)
* Edit panels (name, height, display options)
* Edit sources (name, color, query, panel)
Each change is effective as soon as you type it, but won't persist a restart if you don't "save" it. Just close and reopen if you mess something up!

## Features
* parse JSON apis with [JQL syntax](https://github.com/yamafaktory/jql)
* embedded SQLite, no need for external database
* import/export metrics data to/from CSV
* split data from 1 fetch to many metrics
* customize source color and name, toggle them (visibility or fetching)
* customize panels (size, span, offset)
* reduce data points with average or sampling
* per-source query interval
* light/dark mode
* log panel endlessly tracking errors
* tiny performance impact

## Drawbacks
* Log panel has no limit, thus very long runtimes will make it slower
* Being monolithic, this project doesn't scale well with large data needs
* Untested on Windows and MacOS
* No limit on points displayed might slow down the UI, use the `reduce` feature
* All fields are editable at the same time

# Installation
`cargo build --release`, then drop it in your `~/.local/bin`. Done, have fun hoarding data!
