# Caching in PSST

PSST now includes comprehensive caching for playlist, song, and artist data to improve performance and reduce API calls to Spotify.

## Overview

The caching system consists of two main components:

1. **Core Cache** (`psst-core/src/cache.rs`) - Caches audio files, track metadata, and audio keys
2. **Web API Cache** (`psst-gui/src/webapi/cache.rs`) - Caches API responses for playlists, tracks, artists, and other data

## What's Cached

### Web API Cache
- **Tracks**: Individual track data, user's saved tracks, top tracks
- **Playlists**: Playlist metadata and track lists
- **Artists**: Artist info, albums, top tracks, related artists
- **Images**: Album covers and artist images (in-memory LRU + disk cache)
- **User Data**: User profile, playlists, saved content

### Core Cache
- **Audio Files**: Encrypted audio content
- **Track Metadata**: Protobuf track data
- **Audio Keys**: Decryption keys for audio files
- **Episodes**: Podcast episode metadata

## Cache Implementation

### Web API Cache Structure
```
cache/
├── track/           # Individual track data
├── playlist/        # Playlist metadata
├── playlist-tracks/ # Playlist track lists
├── artist/          # Artist metadata
├── artist-albums/   # Artist album lists
├── artist-top-tracks/ # Artist top tracks
├── related-artists/ # Related artists
├── artist-info/     # Artist biography and stats
├── saved-tracks/    # User's saved tracks
├── user-top-tracks/ # User's top tracks
├── playlists/       # User's playlists
├── images/          # Album covers and images
└── TrackLines/      # Song lyrics
```

### Cache Invalidation

The cache automatically invalidates when:
- Tracks are added/removed from playlists
- User saves/unsaves tracks
- Playlist details are modified

Manual cache clearing is available in the Preferences UI.

## Performance Benefits

- **Faster Loading**: Cached data loads instantly instead of making API calls
- **Reduced API Usage**: Fewer requests to Spotify's servers
- **Offline Capability**: Basic data available when offline
- **Better UX**: Smoother navigation and faster search results

## Cache Management

### Preferences UI
- View cache statistics (size, entry count)
- Clear core cache (audio files, metadata)
- Clear Web API cache (playlists, tracks, artists)
- Monitor cache usage

### Programmatic Access
```rust
// Get cache statistics
let stats = WebApi::global().get_cache_stats();

// Clear specific caches
WebApi::global().clear_playlist_cache("playlist_id");
WebApi::global().clear_artist_cache("artist_id");
WebApi::global().clear_track_cache("track_id");
WebApi::global().clear_user_cache();

// Clear all Web API cache
WebApi::global().clear_web_api_cache()?;
```

## Cache Configuration

Cache location is determined by the application's cache directory:
- **macOS**: `~/Library/Caches/psst/`
- **Linux**: `~/.cache/psst/`
- **Windows**: `%LOCALAPPDATA%\psst\cache\`

## Cache Size Management

- **Image Cache**: 256 entries in-memory LRU cache
- **Disk Cache**: Unlimited size (managed by user)
- **Automatic Cleanup**: Cache can be cleared via Preferences

## Testing

Run cache tests with:
```bash
cargo test --package psst-gui --lib webapi::cache::tests
```

## Future Enhancements

- Cache expiration based on data freshness
- Automatic cache size management
- Cache compression for large datasets
- Background cache warming
- Cache sharing between multiple PSST instances 