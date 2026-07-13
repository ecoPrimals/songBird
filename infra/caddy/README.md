# Caddy Configuration — songBird Compositions

## footPrint GIS Proxy

`footprint-gis-proxy.Caddyfile` defines a reusable snippet `(footprint_gis_proxy)` that
proxies 10 GIS upstream services with TLS handled natively by Caddy.

### Usage in site Caddyfile

```caddyfile
import /path/to/footprint-gis-proxy.Caddyfile

primals.eco {
    # footPrint static assets (built Vite app)
    handle_path /footprint/* {
        root * /srv/footprint/dist
        file_server
    }

    # footPrint GIS external proxy
    import footprint_gis_proxy

    # songBird federation / drawbridge (internal)
    handle /api/* {
        reverse_proxy localhost:7780
    }
}
```

### Service Mapping

| Path Segment | Upstream |
|---|---|
| `/footprint/ext/overpass/` | `https://overpass-api.de` |
| `/footprint/ext/fema/` | `https://hazards.fema.gov` |
| `/footprint/ext/arcgis1/` | `https://services1.arcgis.com` |
| `/footprint/ext/arcgis2/` | `https://services2.arcgis.com` |
| `/footprint/ext/nominatim/` | `https://nominatim.openstreetmap.org` |
| `/footprint/ext/usgs/` | `https://epqs.nationalmap.gov` |
| `/footprint/ext/nrcs/` | `https://sdmdataaccess.sc.egov.usda.gov` |
| `/footprint/ext/michigan/` | `https://gisagocss.state.mi.us` |
| `/footprint/ext/mcgi/` | `https://gisp.mcgi.state.mi.us` |
| `/footprint/ext/eastlansing/` | `https://gis2.cityofeastlansing.com` |

### Why Caddy over songBird drawbridge for production

- Caddy's `reverse_proxy` with TLS transport handles upstream HTTPS natively
- No dependency on `SOCKET-DIR-UNIFY` (biomeOS) for TLS delegation
- ACME HTTP-01 for client-facing cert (once DNS `A live → 157.230.3.183` is set)
- Connection pooling and HTTP/2 upstream built-in
- `handle_path` strips the prefix automatically — no path rewriting logic needed
