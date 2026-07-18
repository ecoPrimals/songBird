# Caddy Configuration — songBird Compositions

## Subdomain Routing Standard (Wave 150d)

All compositions use `prefix.primals.eco` subdomains. Path-based routing
(`primals.eco/path/`) is **prohibited**. Cloudflare wildcard DNS resolves
`*.primals.eco` to golgiBody — only a Caddy block is needed per composition.

## footPrint GIS Proxy

`footprint-gis-proxy.Caddyfile` defines a reusable snippet `(footprint_gis_proxy)` that
proxies 10 GIS upstream services with TLS handled natively by Caddy.

### Production Caddyfile (golgiBody)

```caddyfile
import /etc/caddy/snippets/footprint-gis-proxy.Caddyfile

footprint.primals.eco {
    # Option A: let footPrint Express handle everything (simplest)
    reverse_proxy 10.13.37.2:8090

    # Option B: Caddy does GIS proxying directly (production optimization)
    # import footprint_gis_proxy
    # handle {
    #     reverse_proxy 10.13.37.2:8090
    # }

    header Content-Security-Policy "img-src 'self' *.arcgisonline.com *.tile.openstreetmap.org data: blob:;"
}

webb.primals.eco {
    reverse_proxy 10.13.37.6:8090
}

sporeprint.primals.eco {
    reverse_proxy localhost:1111
}

primals.eco {
    redir https://sporeprint.primals.eco{uri} permanent
}
```

### GIS Service Mapping

| Path (under subdomain) | Upstream |
|---|---|
| `/ext/overpass/` | `https://overpass-api.de` |
| `/ext/fema/` | `https://hazards.fema.gov` |
| `/ext/arcgis1/` | `https://services1.arcgis.com` |
| `/ext/arcgis2/` | `https://services2.arcgis.com` |
| `/ext/nominatim/` | `https://nominatim.openstreetmap.org` |
| `/ext/usgs/` | `https://epqs.nationalmap.gov` |
| `/ext/nrcs/` | `https://sdmdataaccess.sc.egov.usda.gov` |
| `/ext/michigan/` | `https://gisagocss.state.mi.us` |
| `/ext/mcgi/` | `https://gisp.mcgi.state.mi.us` |
| `/ext/eastlansing/` | `https://gis2.cityofeastlansing.com` |

## Science API Proxy

`science-api-proxy.Caddyfile` defines a snippet `(science_api_proxy)` for 6 science
database APIs (NCBI, PubChem, BLAST, UniProt, PDB, AlphaFold).

### Science Service Mapping

| Path | Upstream |
|---|---|
| `/science/ext/ncbi/` | `https://eutils.ncbi.nlm.nih.gov` |
| `/science/ext/pubchem/` | `https://pubchem.ncbi.nlm.nih.gov` |
| `/science/ext/blast/` | `https://blast.ncbi.nlm.nih.gov` |
| `/science/ext/uniprot/` | `https://rest.uniprot.org` |
| `/science/ext/pdb/` | `https://data.rcsb.org` |
| `/science/ext/alphafold/` | `https://alphafold.ebi.ac.uk` |

## tideGlass Pharmacogenomics Proxy

`tideglass-pharma-proxy.Caddyfile` defines a snippet `(tideglass_pharma_proxy)` for 4
pharmacogenomics/computational biology APIs.

### tideGlass Service Mapping

| Path | Upstream | Data |
|---|---|---|
| `/tideglass/ext/lincs/` | `https://maayanlab.cloud/sigcom-lincs` | L1000 perturbational signatures |
| `/tideglass/ext/geo/` | `https://eutils.ncbi.nlm.nih.gov` | Gene Expression Omnibus |
| `/tideglass/ext/chembl/` | `https://www.ebi.ac.uk/chembl/api` | Drug bioactivity data |
| `/tideglass/ext/nf/` | `https://portal-api.synapse.org` | NF Data Portal (Synapse) |

## Drawbridge Allowlist (dev/test without Caddy)

GIS APIs:
```bash
export SONGBIRD_DRAWBRIDGE_EXTERNAL_ALLOWLIST="overpass=https://overpass-api.de,fema=https://hazards.fema.gov,arcgis1=https://services1.arcgis.com,arcgis2=https://services2.arcgis.com,nominatim=https://nominatim.openstreetmap.org,usgs=https://epqs.nationalmap.gov,nrcs=https://sdmdataaccess.sc.egov.usda.gov,michigan=https://gisagocss.state.mi.us,mcgi=https://gisp.mcgi.state.mi.us,eastlansing=https://gis2.cityofeastlansing.com"
```

Science APIs:
```bash
export SONGBIRD_DRAWBRIDGE_EXTERNAL_ALLOWLIST="ncbi=https://eutils.ncbi.nlm.nih.gov,pubchem=https://pubchem.ncbi.nlm.nih.gov,blast=https://blast.ncbi.nlm.nih.gov,uniprot=https://rest.uniprot.org,pdb=https://data.rcsb.org,alphafold=https://alphafold.ebi.ac.uk"
```

tideGlass APIs:
```bash
export SONGBIRD_DRAWBRIDGE_EXTERNAL_ALLOWLIST="lincs=https://maayanlab.cloud/sigcom-lincs,geo=https://eutils.ncbi.nlm.nih.gov,chembl=https://www.ebi.ac.uk/chembl/api,nf=https://portal-api.synapse.org"
```

The `songbird_types::defaults::bonds` module provides these as constants
(`GIS_BONDS`, `SCIENCE_BONDS`, `TIDEGLASS_BONDS`, `ALL_BONDS`) with
`format_allowlist()` for programmatic use.

## Architecture: songBird = Inner Membrane Port Solver

```
User → Cloudflare DNS (*.primals.eco → golgiBody)
  → Cloudflare CDN (outer membrane firebreak)
    → Caddy on golgiBody (TLS termination, Host-header routing)
      → reverse_proxy over WireGuard to target gate
        → songBird drawbridge :7780 (capability → port resolution)
          → Local service (footPrint:8090, esotericWebb:8090, etc.)
```

**Production**: Caddy handles external HTTPS proxying directly via these snippets.
**Dev/test**: songBird drawbridge handles proxying via `SONGBIRD_DRAWBRIDGE_EXTERNAL_ALLOWLIST`.

## Why Caddy over songBird drawbridge for production

- Caddy's `reverse_proxy` with TLS transport handles upstream HTTPS natively
- Connection pooling and HTTP/2 upstream built-in
- ACME HTTP-01 for client-facing cert (Cloudflare proxied)
- `handle_path` strips the prefix automatically — no path rewriting logic needed
- songBird drawbridge remains the dev/test fallback (no Caddy dependency required)
