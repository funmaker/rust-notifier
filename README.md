# rust-notifier
Server in Rust for generating notifications from RSS, 4chan threads, YouTube subscriptions and more.

Clients:
- [KLWP Home Screen](https://github.com/funmaker/home-screen)
- [Mumble Plugin](https://github.com/Luksor/mumo-notifier) (Outdated)

## Features
- [X] RSS Server support
- [X] RSS Reader support
- [X] 4chan support
- [X] YouTube Subscriptions support
- [X] Vinesauce support
- [ ] Websocket support
- [ ] Twitch support

## Configuration

[Config](#Config) is stored in `config.json` in the working directory.
See [config_example.json](config_example.json) for example configuration.

## API Reference

`web` interface hosts HTTP server providing read only access to data.

Only one route is currently supported:

### GET /feeds

| Field   | Type   | Comment |
| ------- | ------ | ------- |
| filter  | String | Optional. Regex. Only feeds with name matching filter will be returned. If absent, all feeds will be returned. |
| format  | `rss` or `json` | Optional. Specified output format. `json` by default. |
| flat    | Boolean | Optional. Flattens output into single Feed. `true` by default. |

If you choose `rss` format, server will respond with RSS 2.0 feed.
Otherwise server will respond with either single [Feed](#Feed)

### Types
#### Feed
| Field   | Type   | Comment |
| ------- | ------ | ------- |
| status  | Array of [Entries](#Entry) | "Fixed" notifications that disappear after some time. eg. 4chan threads, live streams, etc. |
| errors  | Array of [Entries](#Entry) | Error notifications indicating problems during fetching. |
| notifications | Array of [Entries](#Entry) | Regular notifications. eg. RSS entries or youtube videos |

#### Entry
| Field   | Type   | Comment |
| ------- | ------ | ------- |
| title   | String | Primary content |
| guid    | String | ID unique for specific feed |
| feedName | String | Name of the parent Feed |
| description | String | Optional. Secondary content |
| link | String | Optional. |
| color | String | Optional. eg. "#FF00FF" |
| imageURL | String | Optional. Image thumbnail or preview. |
| timestamp | Integer | Optional. JS timestamp, amount of milliseconds since UNIX epoch. |
| extra | Object | Optional. Additional data. |

#### Config
| Field   | Type   | Comment |
| ------- | ------ | ------- |
| fetchIntervalSecs | Number | Interval at which new notifications will be fetched |
| feeds | Map of [FeedConfig](#FeedConfig) | Keys represent the name of the feed |
| providers | Map of [ProviderConfig](#ProviderConfig) | Keys represent the name of the provider |
| interfaces | Map of [InterfaceConfig](#InterfaceConfig) | Keys represent the name of the interface |

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
| filter | String | Regex. OPs matching will be returned. |

##### Vinesauce
| Field   | Type   | Comment |
| ------- | ------ | ------- |
| channels  | Array of Strings | Names of channels to subscribe, eg: ["vinesauce", "vargskelethor"] |

#### ProviderConfig
| Field   | Type   | Comment |
| ------- | ------ | ------- |
| enabled  | Boolean | Enables specific provider |

Additionally `youtube` provider requires following fields:

| Field   | Type   | Comment |
| ------- | ------ | ------- |
| apiKey  | String | Youtube Data API v3 key |

#### InterfaceConfig
| Field   | Type   | Comment |
| ------- | ------ | ------- |
| enabled  | Boolean | Enables specific provider |

Additionally `web` provider requires following fields:

| Field   | Type   | Comment |
| ------- | ------ | ------- |
| port    | Number | Port on which web interface should be hosted |
| rest    | Boolean | Enabled REST API (doesn't do anything yet) |
| websocket | Boolean | Enabled WebSocket API (doesn't do anything yet) |
| rss     | Boolean | Enabled RSS API (doesn't do anything yet) |
