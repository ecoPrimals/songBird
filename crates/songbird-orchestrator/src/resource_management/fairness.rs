//! Fairness Calculator
//!
//! Calculates fair share allocations based on:
//! - Historical usage
//! - User priorities
//! - System capacity

use super::{ResourceAmount, ResourceType};
use crate::task_lifecycle::UserId;
use std::collections::HashMap;

/// Fair share calculation
#[derive(Debug, Clone)]
pub struct FairShare {
    pub user_id: UserId,
    pub resource_type: ResourceType,
    pub fair_share: ResourceAmount,
    pub current_usage: ResourceAmount,
    pub ratio: f64, // usage / fair_share
}

/// Fairness calculator
pub struct FairnessCalculator;

impl FairnessCalculator {
    /// Calculate fair shares for all users
    #[must_use]
    pub fn calculate_fair_shares(
        total_capacity: &HashMap<ResourceType, ResourceAmount>,
        user_weights: &HashMap<UserId, f64>,
        current_usage: &HashMap<UserId, HashMap<ResourceType, ResourceAmount>>,
    ) -> Vec<FairShare> {
        let mut results = Vec::new();

        // Sum all weights
        let total_weight: f64 = user_weights.values().sum();

        if total_weight == 0.0 {
            return results;
        }

        // Calculate fair share for each user and resource type
        for (user_id, weight) in user_weights {
            for (resource_type, capacity) in total_capacity {
                // Fair share = (user_weight / total_weight) * capacity
                let share_ratio = weight / total_weight;
                let fair_share_value = capacity.value * share_ratio;

                let fair_share = ResourceAmount {
                    value: fair_share_value,
                    unit: capacity.unit,
                };

                // Get current usage
                let usage = current_usage
                    .get(user_id)
                    .and_then(|u| u.get(resource_type))
                    .copied()
                    .unwrap_or_else(|| ResourceAmount::zero(capacity.unit));

                // Calculate ratio
                let ratio = if fair_share_value > 0.0 {
                    usage.value / fair_share_value
                } else {
                    0.0
                };

                results.push(FairShare {
                    user_id: user_id.clone(),
                    resource_type: *resource_type,
                    fair_share,
                    current_usage: usage,
                    ratio,
                });
            }
        }

        results
    }

    /// Identify users who are over their fair share
    #[must_use]
    pub fn identify_over_usage(fair_shares: &[FairShare], threshold: f64) -> Vec<FairShare> {
        fair_shares.iter().filter(|fs| fs.ratio > threshold).cloned().collect()
    }

    /// Identify users who are under their fair share
    #[must_use]
    pub fn identify_under_usage(fair_shares: &[FairShare], threshold: f64) -> Vec<FairShare> {
        fair_shares.iter().filter(|fs| fs.ratio < threshold).cloned().collect()
    }

    /// Calculate dominant resource fairness (DRF)
    /// Returns the dominant resource ratio for each user
    #[must_use]
    pub fn dominant_resource_fairness(
        fair_shares: &[FairShare],
    ) -> HashMap<UserId, (ResourceType, f64)> {
        let mut drf_map = HashMap::new();

        // Group by user
        let mut by_user: HashMap<UserId, Vec<&FairShare>> = HashMap::new();
        for fs in fair_shares {
            by_user.entry(fs.user_id.clone()).or_default().push(fs);
        }

        // Find dominant resource for each user
        for (user_id, user_shares) in by_user {
            if let Some(dominant) = user_shares
                .iter()
                .max_by(|a, b| a.ratio.partial_cmp(&b.ratio).unwrap_or(std::cmp::Ordering::Equal))
            {
                drf_map.insert(user_id, (dominant.resource_type, dominant.ratio));
            }
        }

        drf_map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource_management::ResourceUnit;
    use crate::task_lifecycle::UserId;

    #[test]
    fn test_fair_share_calculation() {
        let mut capacity = HashMap::new();
        capacity.insert(ResourceType::Cpu, ResourceAmount::new(10.0, ResourceUnit::Cores));

        let mut weights = HashMap::new();
        weights.insert(UserId::from("alice"), 2.0);
        weights.insert(UserId::from("bob"), 1.0);

        let mut usage = HashMap::new();
        let mut alice_usage = HashMap::new();
        alice_usage.insert(ResourceType::Cpu, ResourceAmount::new(5.0, ResourceUnit::Cores));
        usage.insert(UserId::from("alice"), alice_usage);

        let fair_shares = FairnessCalculator::calculate_fair_shares(&capacity, &weights, &usage);

        // Alice has weight 2, Bob has weight 1 (total 3)
        // Alice's fair share = 2/3 * 10 = 6.67
        // Bob's fair share = 1/3 * 10 = 3.33

        let alice_share = fair_shares.iter().find(|fs| fs.user_id.as_str() == "alice").unwrap();

        assert!((alice_share.fair_share.value - 6.67).abs() < 0.1);
        assert_eq!(alice_share.current_usage.value, 5.0);
        assert!((alice_share.ratio - 0.75).abs() < 0.1); // 5.0 / 6.67
    }

    #[test]
    fn test_identify_over_usage() {
        let mut capacity = HashMap::new();
        capacity.insert(ResourceType::Cpu, ResourceAmount::new(10.0, ResourceUnit::Cores));

        let mut weights = HashMap::new();
        weights.insert(UserId::from("alice"), 1.0);
        weights.insert(UserId::from("bob"), 1.0);

        let mut usage = HashMap::new();

        let mut alice_usage = HashMap::new();
        alice_usage.insert(
            ResourceType::Cpu,
            ResourceAmount::new(8.0, ResourceUnit::Cores), // Over fair share
        );
        usage.insert(UserId::from("alice"), alice_usage);

        let mut bob_usage = HashMap::new();
        bob_usage.insert(
            ResourceType::Cpu,
            ResourceAmount::new(2.0, ResourceUnit::Cores), // Under fair share
        );
        usage.insert(UserId::from("bob"), bob_usage);

        let fair_shares = FairnessCalculator::calculate_fair_shares(&capacity, &weights, &usage);

        let over_usage = FairnessCalculator::identify_over_usage(&fair_shares, 1.0);

        assert_eq!(over_usage.len(), 1);
        assert_eq!(over_usage[0].user_id.as_str(), "alice");
    }

    #[test]
    fn test_dominant_resource_fairness() {
        let fair_shares = vec![
            FairShare {
                user_id: UserId::from("alice"),
                resource_type: ResourceType::Cpu,
                fair_share: ResourceAmount::new(5.0, ResourceUnit::Cores),
                current_usage: ResourceAmount::new(4.0, ResourceUnit::Cores),
                ratio: 0.8,
            },
            FairShare {
                user_id: UserId::from("alice"),
                resource_type: ResourceType::Memory,
                fair_share: ResourceAmount::new(8192.0, ResourceUnit::Megabytes),
                current_usage: ResourceAmount::new(7000.0, ResourceUnit::Megabytes),
                ratio: 0.85, // Dominant
            },
        ];

        let drf = FairnessCalculator::dominant_resource_fairness(&fair_shares);

        let alice_drf = drf.get(&UserId::from("alice")).unwrap();
        assert_eq!(alice_drf.0, ResourceType::Memory);
        assert!((alice_drf.1 - 0.85).abs() < 0.01);
    }

    #[test]
    fn test_equal_weights_equal_shares() {
        let mut capacity = HashMap::new();
        capacity.insert(ResourceType::Cpu, ResourceAmount::new(12.0, ResourceUnit::Cores));

        let mut weights = HashMap::new();
        weights.insert(UserId::from("alice"), 1.0);
        weights.insert(UserId::from("bob"), 1.0);
        weights.insert(UserId::from("charlie"), 1.0);

        let usage = HashMap::new();

        let fair_shares = FairnessCalculator::calculate_fair_shares(&capacity, &weights, &usage);

        // Each should get 4.0 cores (12 / 3)
        for share in fair_shares {
            assert!((share.fair_share.value - 4.0).abs() < 0.1);
        }
    }
}
