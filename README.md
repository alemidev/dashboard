# dashboard
A data aggregating dashboard, capable of periodically fetching, parsing, archiving and plotting data.

## Features

## Usage
This program will work on a database stored in `$HOME/.local/share/dashboard.db`. By default, nothing will be shown.
Start editing your dashboard by toggling edit mode on, and add one or more panels (from top bar).
You can now add sources to your panel(s): put an URL pointing to any REST api, dashboard will make a periodic GET request.
Specify how to access data with "y" fields. A JQL query will be used to parse the json data. A value to fetch X data can also be given, if not specified, current time will be used as X when inserting values.
Done! Edit anything to your pleasure, remember to save after editing to make your changes persist, and leave the dashboard hoarding data.
## Install
idk, `cargo build --release`

