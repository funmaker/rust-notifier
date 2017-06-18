# rust-notifier
Server in Rust for generating notifications from RSS, 4chan threads, youtube subscriptions, twitter and more.
Clients: [https://github.com/Luksor/mumo-notifier](Mumble Plugin)

## Features
- [X] Websocket support
- [X] TCP support
- [X] RSS support
- [X] 4chan support
- [X] YouTube Subscriptions support
- [X] Jadisco.pl support
- [X] Vinesauce support
- [ ] Twitch support

## API Reference
All requests and responses are Json objects.
### Types
#### Feed

| Field   | Type   | Comment |
| ------- | ------ | ------- |
| status  | Array of Entries | "Fixed" notifications that disappear after some time. eg. 4chan threads or twitch streams |
| notifications | Array of Entries | Regular notifications. eg. RSS entries or youtube videos |

#### Entry

| Field   | Type   | Comment |
| ------- | ------ | ------- |
| title   | String |  |
| guid    | String | Globally Unique IDntifier |
| feedName | String | Name of the parent Feed |
| error | Bool |  |
| description | String | Optional |
| link | String | Optional |
| color | String | Optional. eg. "#FF00FF" |
| imageURL | String | Optional |
| timestamp | Integer | Optional |
| extra | Object | Optional. Additional data |

#### FeedConfig

| Field   | Type   | Comment |
| ------- | ------ | ------- |
| provider | String | Name of the feed provider to use. eg. "rss", "chan" |
| providerData | Any | Additional data for provider, see below |
| color | String | Optional. Default color for entries |

#### Provider specific Config
##### RSS
String value of the URL of RSS feed.
##### YouTube Subscriptions
String value of YouTube Channel's ID.
##### 4chan

| Field   | Type   | Comment |
| ------- | ------ | ------- |
| boards  | Array of Strings | Boards to search |
| filters | Array of Strings | Array of regex rules. OPs matching any of these will be returned. |

##### Vinesauce

| Field   | Type   | Comment |
| ------- | ------ | ------- |
| channels  | Array of Strings | Names of channels to subscribe, eg: ["vinesauce", "vargskelethor"] |

### Error
| Field   | Type   | Comment |
| ------- | ------ | ------- |
| error | String | Error description |

### Fetching
#### Request

| Field   | Type   | Comment |
| ------- | ------ | ------- |
| command | String | Always "fetch" |
| flat    | bool   | If true, merge all Entries from all Feeds into one Feed |
| feeds   | Array of Strings | Array of regex rules. Feeds matcheing any of these will be returned. |

#### Response

| Field   | Type   | Comment |
| ------- | ------ | ------- |
| feeds | Object | Map of feeds, where key is the name of a Feed |

#### Response(flat = true)

| Field   | Type   | Comment |
| ------- | ------ | ------- |
| status  | Array of Entries | "Fixed" notifications that dissapear after some time. eg. 4chan threads or twitch streams |
| notifications | Array of Entries | Regular notifications. eg. RSS entries or youtube videos |


### Listing Feeds
#### Request

| Field   | Type   | Comment |
| ------- | ------ | ------- |
| command | String | Always "list" |

#### Response

| Field   | Type   | Comment |
| ------- | ------ | ------- |
| feeds | Object | Map of FeedConfigs, where key is the name of a Feed |


### Adding Feeds
#### Request

| Field   | Type   | Comment |
| ------- | ------ | ------- |
| command | String | Always "add" |
| feedName | String | Name of new feed |
| entry | FeedConfig | Configuration of new feed |

#### Response

| Field   | Type   | Comment |
| ------- | ------ | ------- |
| feedName | String | Name of new feed |

### Removing Feeds
#### Request

| Field   | Type   | Comment |
| ------- | ------ | ------- |
| command | String | Always "remove" |
| feedName | String | Name of a feed to remove |

#### Response

| Field   | Type   | Comment |
| ------- | ------ | ------- |
| feedName | String | Name of a removed feed |

### Update Mode
#### Request

| Field   | Type   | Comment |
| ------- | ------ | ------- |
| command | String | Always "update" |

#### Response (will be send at fixed intervals)

| Field   | Type   | Comment |
| ------- | ------ | ------- |
| status  | Array of Entries | New statuses |
| notifications | Array of Entries | New notifications |
