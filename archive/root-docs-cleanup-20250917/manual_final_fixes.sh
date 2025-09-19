#!/bin/bash

echo "Applying final manual fixes for compilation success..."

# Fix specific files that need import semicolons
files=(
    "crates/songbird-types/src/config/api.rs"
    "crates/songbird-types/src/config/communication.rs"
    "crates/songbird-types/src/config/environment.rs"
    "crates/songbird-types/src/config/federation.rs"
    "crates/songbird-types/src/config/gaming.rs"
    "crates/songbird-types/src/config/health.rs"
    "crates/songbird-types/src/config/migration.rs"
    "crates/songbird-types/src/config/network.rs"
    "crates/songbird-types/src/config/orchestration.rs"
    "crates/songbird-types/src/config/security.rs"
    "crates/songbird-types/src/config/unified.rs"
    "crates/songbird-types/src/errors.rs"
    "crates/songbird-types/src/health.rs"
    "crates/songbird-types/src/primal.rs"
    "crates/songbird-types/src/response.rs"
    "crates/songbird-types/src/service.rs"
    "crates/songbird-types/src/traits.rs"
    "crates/songbird-types/src/types.rs"
)

for file in "${files[@]}"; do
    if [ -f "$file" ]; then
        # Fix import statements
        sed -i 's/use serde:{Deserialize, Serialize}$/use serde:{Deserialize, Serialize};/' "$file"
        sed -i 's/use chrono:{DateTime, Utc}$/use chrono:{DateTime, Utc};/' "$file"
        sed -i 's/use crate::service:{CanonicalServiceEndpoint, CanonicalServiceInfo}$/use crate::service:{CanonicalServiceEndpoint, CanonicalServiceInfo};/' "$file"
        echo "Fixed imports in $file"
    fi
done

# Fix function return expressions in errors.rs
sed -i 's/        };$/        }/' crates/songbird-types/src/errors.rs

echo "All manual fixes applied!" 