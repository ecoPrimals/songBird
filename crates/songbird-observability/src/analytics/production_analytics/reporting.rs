// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use super::{
    AnalyticsInsight, AnomalySeverity, InsightType, ProductionAnalyticsEngine, TrendAnalysis,
    TrendDirection,
};
use chrono::Utc;
use songbird_types::SongbirdResult;
use tracing::info;

impl ProductionAnalyticsEngine {
    /// Generate analytics insights
    ///
    /// # Errors
    ///
    /// Returns an error if underlying trend or prediction calls fail.
    pub async fn generate_insights(
        &self,
        metric_name: &str,
    ) -> SongbirdResult<Vec<AnalyticsInsight>> {
        let mut insights = Vec::new();

        if let Ok(trend) = self.analyze_trends(metric_name, 20).await {
            insights.push(AnalyticsInsight {
                insight_type: InsightType::Trend,
                title: format!("Trend Analysis for {metric_name}"),
                description: format!(
                    "Metric shows {:?} trend with {:.1}% strength",
                    trend.direction,
                    trend.strength * 100.0
                ),
                severity: if trend.strength > 0.8 {
                    AnomalySeverity::High
                } else {
                    AnomalySeverity::Medium
                },
                recommendation: Self::generate_trend_recommendation(&trend),
                generated_at: Utc::now(),
            });
        }

        if let Ok(prediction) =
            self.predict_metric(metric_name, self.config.prediction_horizon).await
        {
            insights.push(AnalyticsInsight {
                insight_type: InsightType::Prediction,
                title: format!("Prediction for {metric_name}"),
                description: format!(
                    "Predicted value: {:.2} (confidence: {:.1}%)",
                    prediction.predicted_value,
                    prediction.confidence * 100.0
                ),
                severity: AnomalySeverity::Low,
                recommendation: "Monitor predicted values for planning".to_string(),
                generated_at: Utc::now(),
            });
        }

        info!("Generated {} insights for metric: {}", insights.len(), metric_name);
        Ok(insights)
    }

    pub(crate) fn generate_trend_recommendation(trend: &TrendAnalysis) -> String {
        match trend.direction {
            TrendDirection::Increasing => {
                if trend.strength > 0.8 {
                    "Strong upward trend detected. Consider scaling resources or investigating cause."
                        .to_string()
                } else {
                    "Gradual increase observed. Monitor for continued growth.".to_string()
                }
            }
            TrendDirection::Decreasing => {
                if trend.strength > 0.8 {
                    "Strong downward trend detected. Investigate potential issues or optimization opportunities."
                        .to_string()
                } else {
                    "Gradual decrease observed. Monitor for continued decline.".to_string()
                }
            }
            TrendDirection::Stable => {
                "Metric remains stable. No immediate action required.".to_string()
            }
            TrendDirection::Volatile => {
                "High volatility detected. Consider investigating root causes.".to_string()
            }
        }
    }
}
