# Simple Page View Counter

Single binary, local database solution for counting page views on a static website.

## Features

- Tracks total visits each URL had.
- Tracks unique visits each URL had.
- Can save the user agent of the visitor.
- Can save the IP of the visitor.
- Rejects requests coming from URLs other than the ones in the allowed list.
  Unauthorised calls get logged as warings in the journal
  (can be disabled by running like this: `SPVC_LOG=spvc=error spvc [options]`).

## How it works

It works by using the `referer` header when the server API is called from within the current page,
which is then logged with a datetime and a few other things based on configuration.
A small cookie is also set to be able to count unique visitors.

## Website setup

All the website side needs is this line in the header of each page you want to track.

```html
<script referrerpolicy="same-origin" src="https://mywebsite.com/spvc/api/log_visitor"></script>
```

## Server setup

This is how I have it set up.

### `nginx`

This assumes you plan to set this program on the same machine and for the same website.
Add the following `location` to your `server`:

```nginx
location /spvc/ {
    proxy_pass http://127.0.0.1:7782/;
    proxy_redirect http:// https://;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Real-IP $http_x_real_ip;
}
```

Be careful with the forward slashes, since they **do** make a difference.
Read [this](https://tarunlalwani.com/post/nginx-proxypass-server-paths/) to learn more.

You can remove both `proxy_set_header`s if you're not logging the IP of the visitors.

### `systemd`

Run the `spvc` executable as a `systemd` service. Here is an example of `spvc.service`:

```ini
[Unit]
Description=SPVC service
After=network.target
StartLimitIntervalSec=0

[Service]
Type=simple
User=root
WorkingDirectory=/root/spvc
ExecStart=/root/spvc/spvc -au https://mywebsite.com http://mywebsite.com
Restart=always
RestartSec=1

[Install]
WantedBy=multi-user.target
```

Either save it or `ln -sf` it to `/etc/systemd/system/spvc.service`.
Then run `systemctl daemon-reload` to load in the new service (must reload each time you make a change to it).
Finally, enable with `systemctl enable spvc`.
Remember that you can see the logs with `journalctl -u spvc`.

### `spvc`

Here's the help message:

```python
USAGE:
    spvc [OPTIONS] <ALLOWED_URLS>...

ARGS:
    <ALLOWED_URLS>...    Checks the start of each URL against this list

OPTIONS:
    -a, --address <ADDRESS>    Set a socket address to listen to [default: 127.0.0.1:7782]
    -d, --db-path <DB_PATH>    Path to the SQLite database [default: spvc.db]
    -h, --help                 Print help information
    -i, --save-ip              Enable saving the visitor's IP address
    -u, --save-user-agent      Enable saving the visitor's user agent
    -V, --version              Print version information
```

There is no configuration file, so you set these options in the `ExecStart` field of `spvc.service`.

### Running for multiple websites on the same machine

You can either have multiple `systemd` services running `spvc` for each site,
or have one main `spvc` sub/domain that you call to from all your websites using
`referrerpolicy="no-referrer-when-downgrade"`.
The choice is yours.

## API

At the moment there is only one API endpoint: `log_visitor`.
I plan to add more endpoints to be able to retrieve how many times a page was viewed in total,
by unique visitors, etc.

## How to view the data?

Right now there's no way other than to simply open the SQLite database itself and query manually.
I plan to add some tables accessible through the website's `https://mywebsite.com/spvc/` path
but for now you can use a query like this to check how many views each logged URL has in descending order:

```sql
SELECT urls.url, count(visits.url) AS visits FROM visits INNER JOIN urls ON visits.url = urls.id GROUP BY visits.url ORDER BY visits DESC;
```

Take a look at the `schema.sql` file to see how the database is structured.

## Do you need to ask the user for consent?

From my research and asking people with experience,
there should be no issue logging everything indefinitely except for the IP address.
The small cookie to identify the visitor doesn't require consent, either.

When it comes to the IP address,
it seems to be no problem to log it for up to a month if it's "for the backend system and intrusion protection",
so I'm not quite sure if this program qualifies.
I will use the IP just to see where the visitors come from geographically,
so I plan to add a feature that will generate a monthly report,
then delete the previous month's IP data.
Hopefully this falls under that.
Please contact me if you can prove me otherwise.

When it comes to the user agent, I only added the ability to log it because it was easy.
I don't really plan to do anything with it at the moment.
Could be useful for seeing the percentage of the devices used to visit your site.

Either way, it's safer to mention in the ToS that you're logging page views with non-identifiable data
even if you don't log the IP.
This is still way less spooky that Google Analytics, anyway.
Of course, if you ask for consent with a pop-up of some sort and they accept,
you are allowed to store the IP indefinitely.
I do have some ideas for exposing an API where the user can opt out even of the 30 day logging by themselves.

## Contribute

Pull requests are welcomed, but please follow my general coding style
(which mostly amounts to running `rustfmt` on save).

You can contact me most easily through my Discord server with
[discord.gg/D6gmhMUrrH](https://discord.gg/D6gmhMUrrH).

But you can also contact me directly as listed at
[alexchaplinbraz.com/contact](https://alexchaplinbraz.com/contact).

## Legal

MIT License.
