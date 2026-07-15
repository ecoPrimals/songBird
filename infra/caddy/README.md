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

## Science API Proxy

`science-api-proxy.Caddyfile` defines a snippet `(science_api_proxy)` for 6 science
database APIs (NCBI, PubChem, BLAST, UniProt, PDB, AlphaFold).

### Science Service Mapping

| Path Segment | Upstream |
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

| Path Segment | Upstream | Data |
|---|---|---|
| `/tideglass/ext/lincs/` | `https://maayanlab.cloud/sigcom-lincs` | L1000 perturbational signatures |
| `/tideglass/ext/geo/` | `https://eutils.ncbi.nlm.nih.gov` | Gene Expression Omnibus |
| `/tideglass/ext/chembl/` | `https://www.ebi.ac.uk/chembl/api` | Drug bioactivity data |
| `/tideglass/ext/nf/` | `https://portal-api.synapse.org` | NF Data Portal (Synapse) |

### Drawbridge Allowlist (dev/test without Caddy)

Science APIs:
```bash
export SONGBIRD_DRAWBRIDGE_EXTERNAL_ALLOWLIST="ncbi=https://eutils.ncbi.nlm.nih.gov,pubchem=https://pubchem.ncbi.nlm.nih.gov,blast=https://blast.ncbi.nlm.nih.gov,uniprot=https://rest.uniprot.org,pdb=https://data.rcsb.org,alphafold=https://alphafold.ebi.ac.uk"
```

tideGlass pharmacogenomics APIs:
```bash
export SONGBIRD_DRAWBRIDGE_EXTERNAL_ALLOWLIST="lincs=https://maayanlab.cloud/sigcom-lincs,geo=https://eutils.ncbi.nlm.nih.gov,chembl=https://www.ebi.ac.uk/chembl/api,nf=https://portal-api.synapse.org"
```

The `songbird_types::defaults::bonds` module provides these as constants
(`GIS_BONDS`, `SCIENCE_BONDS`, `TIDEGLASS_BONDS`, `ALL_BONDS`) with
`format_allowlist()` for programmatic use.

## Why Caddy over songBird drawbridge for production

- Caddy's `reverse_proxy` with TLS transport handles upstream HTTPS natively
- No dependency on `SOCKET-DIR-UNIFY` (biomeOS) for TLS delegation
- ACME HTTP-01 for client-facing cert (once DNS `A live → 157.230.3.183` is set)
- Connection pooling and HTTP/2 upstream built-in
- `handle_path` strips the prefix automatically — no path rewriting logic needed
