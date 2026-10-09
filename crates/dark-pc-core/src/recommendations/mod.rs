//! V2.5 — Moteur de recommandations (règles déterministes, local, sans IA).

use anyhow::Result;
use chrono::{Duration, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::cleanup::CleanupCategorySummary;
use crate::dashboard;
use crate::db::storage_snapshot_repo;
use crate::disk;
use crate::models::RiskLevel;
use crate::settings;

pub const MAX_RECOMMENDATIONS: usize = 5;

pub mod thresholds {
    pub const STORAGE_INFO_PERCENT: f32 = 75.0;
    pub const STORAGE_WARNING_PERCENT: f32 = 85.0;
    pub const STORAGE_IMPORTANT_PERCENT: f32 = 90.0;
    pub const STORAGE_CRITICAL_PERCENT: f32 = 95.0;

    pub const LARGE_FILES_MIN_COUNT: u64 = 3;
    pub const DUPLICATES_MIN_RECOVERABLE_BYTES: u64 = 100 * 1024 * 1024;
    pub const GROWTH_MIN_BYTES: i64 = 5 * 1024 * 1024 * 1024;
    pub const GROWTH_PERIOD_DAYS: i32 = 7;
    pub const OLD_FILES_DAYS: i64 = 365;
    pub const OLD_FILES_MIN_COUNT: u64 = 10;
    pub const CLEANUP_MIN_BYTES: u64 = 200 * 1024 * 1024;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Warning,
    Important,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Info => "info",
            Severity::Warning => "warning",
            Severity::Important => "important",
        }
    }
}

/// DTO exposé au frontend (complète le squelette `models::Recommendation`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationDto {
    pub id: String,
    pub recommendation_type: String,
    pub severity: String,
    pub title: String,
    pub description: String,
    pub reason: String,
    pub potential_bytes: u64,
    pub action_label: String,
    pub action_route: String,
    pub detected_value: Option<f64>,
    pub threshold: Option<f64>,
    pub period_days: Option<u32>,
    pub mount_path: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationsResult {
    pub recommendations: Vec<RecommendationDto>,
    pub generated_at: String,
}

#[derive(Debug, Clone)]
pub struct EngineContext {
    mount_path: String,
    usage_percent: f32,
    free_bytes: u64,
    _total_bytes: u64,
    insights: dashboard::StorageInsights,
    large_threshold_bytes: u64,
    growth_bytes: Option<i64>,
    growth_period_days: u32,
    old_files_count: u64,
    old_files_bytes: u64,
}

pub fn generate_recommendations(conn: &Connection) -> Result<RecommendationsResult> {
    generate_recommendations_with_cleanup(conn, None)
}

/// Analyse cleanup optionnelle (WalkDir borné) — à n'activer que si l'appelant l'accepte.
pub fn generate_recommendations_with_cleanup(
    conn: &Connection,
    cleanup: Option<&crate::cleanup::CleanupAnalysisResult>,
) -> Result<RecommendationsResult> {
    let ctx = build_context(conn)?;
    let categories = cleanup.map(|c| c.categories.as_slice());
    Ok(RecommendationsResult {
        recommendations: finalize_recommendations(&ctx, categories),
        generated_at: Utc::now().to_rfc3339(),
    })
}

fn finalize_recommendations(
    ctx: &EngineContext,
    cleanup_categories: Option<&[CleanupCategorySummary]>,
) -> Vec<RecommendationDto> {
    let mut items = Vec::new();
    items.extend(rule_storage_usage(ctx));
    items.extend(rule_storage_growth(ctx));
    items.extend(rule_large_files(ctx));
    items.extend(rule_duplicates(ctx));
    items.extend(rule_old_files(ctx));
    items.extend(rule_applications(ctx));
    if let Some(cats) = cleanup_categories {
        items.extend(rule_cleanup(cats));
    }
    dedupe_by_id(&mut items);
    items.sort_by(|a, b| {
        severity_rank(b.severity.as_str())
            .cmp(&severity_rank(a.severity.as_str()))
            .then_with(|| b.potential_bytes.cmp(&a.potential_bytes))
            .then_with(|| a.id.cmp(&b.id))
    });
    items.truncate(MAX_RECOMMENDATIONS);
    items
}

fn build_context(conn: &Connection) -> Result<EngineContext> {
    let volume = disk::volume_stats(None)?;
    let insights = dashboard::get_storage_insights(conn)?;
    let large_threshold_bytes = settings::large_file_threshold_bytes(conn)?;

    let growth = compute_storage_growth(conn, &volume.mount_path, thresholds::GROWTH_PERIOD_DAYS);
    let (old_files_count, old_files_bytes) = old_files_stats(conn, &insights)?;

    Ok(EngineContext {
        mount_path: volume.mount_path.clone(),
        usage_percent: volume.usage_percent,
        free_bytes: volume.available_bytes,
        _total_bytes: volume.total_bytes,
        insights,
        large_threshold_bytes,
        growth_bytes: growth.0,
        growth_period_days: growth.1,
        old_files_count,
        old_files_bytes,
    })
}

fn rule_storage_usage(ctx: &EngineContext) -> Vec<RecommendationDto> {
    let p = ctx.usage_percent;
    if p < thresholds::STORAGE_INFO_PERCENT {
        return Vec::new();
    }

    let (severity, title, description) = if p >= thresholds::STORAGE_CRITICAL_PERCENT {
        (
            Severity::Important,
            "Stockage très saturé",
            "Espace disque critique : agissez bientôt pour éviter les erreurs système.",
        )
    } else if p >= thresholds::STORAGE_IMPORTANT_PERCENT {
        (
            Severity::Important,
            "Stockage presque plein",
            "Libérez de l'espace ou examinez ce qui occupe le plus de place.",
        )
    } else if p >= thresholds::STORAGE_WARNING_PERCENT {
        (
            Severity::Warning,
            "Stockage élevé",
            "Surveillez l'utilisation et planifiez un nettoyage si nécessaire.",
        )
    } else {
        (
            Severity::Info,
            "Utilisation du stockage",
            "Votre disque commence à se remplir.",
        )
    };

    let mount = ctx.mount_path.trim_end_matches('\\');
    vec![make_rec(
        "storage_usage_high",
        "storage_full",
        severity,
        title,
        description.to_string(),
        format!(
            "Le volume {} est utilisé à {:.1} %. Il reste environ {} libres.",
            mount,
            p,
            format_bytes_short(ctx.free_bytes)
        ),
        0,
        "Examiner le stockage",
        "/dashboard",
        Some(f64::from(p)),
        Some(f64::from(thresholds::STORAGE_WARNING_PERCENT)),
        None,
        Some(ctx.mount_path.clone()),
    )]
}

fn rule_storage_growth(ctx: &EngineContext) -> Vec<RecommendationDto> {
    let Some(delta) = ctx.growth_bytes else {
        return Vec::new();
    };
    if delta < thresholds::GROWTH_MIN_BYTES {
        return Vec::new();
    }
    let days = ctx.growth_period_days;
    vec![make_rec(
        "storage_growth",
        "storage_growth",
        Severity::Warning,
        "Hausse récente de l'utilisation",
        format!(
            "Votre espace utilisé a augmenté d'environ {} sur les {} derniers jours.",
            format_bytes_short(delta as u64),
            days
        ),
        format!(
            "Comparaison basée sur les snapshots locaux du volume {}.",
            ctx.mount_path.trim_end_matches('\\')
        ),
        delta.max(0) as u64,
        "Voir l'évolution",
        "/dashboard",
        Some(delta as f64),
        Some(thresholds::GROWTH_MIN_BYTES as f64),
        Some(days),
        Some(ctx.mount_path.clone()),
    )]
}

fn rule_large_files(ctx: &EngineContext) -> Vec<RecommendationDto> {
    if ctx.insights.scan.is_none() {
        return Vec::new();
    }
    let count = ctx.insights.large_files_count;
    if count < thresholds::LARGE_FILES_MIN_COUNT {
        return Vec::new();
    }
    let mb = ctx.large_threshold_bytes / (1024 * 1024);
    vec![make_rec(
        "large_files_detected",
        "large_files",
        if count >= 10 {
            Severity::Warning
        } else {
            Severity::Info
        },
        "Fichiers volumineux détectés",
        format!(
            "{count} fichier(s) dépassent votre seuil configuré ({mb} Mo) dans le dernier scan.",
        ),
        format!(
            "Ensemble, ils représentent environ {} indexés.",
            format_bytes_short(ctx.insights.large_files_total_bytes)
        ),
        ctx.insights.large_files_total_bytes,
        "Examiner les gros fichiers",
        "/large-files",
        Some(count as f64),
        Some(mb as f64),
        None,
        None,
    )]
}

fn rule_duplicates(ctx: &EngineContext) -> Vec<RecommendationDto> {
    if !ctx.insights.has_duplicate_analysis {
        return Vec::new();
    }
    let recoverable = ctx.insights.duplicate_recoverable_bytes;
    if recoverable < thresholds::DUPLICATES_MIN_RECOVERABLE_BYTES {
        return Vec::new();
    }
    let groups = ctx.insights.duplicate_group_count;
    vec![make_rec(
        "duplicates_detected",
        "duplicates",
        Severity::Warning,
        "Doublons détectés",
        format!(
            "{groups} groupe(s) de fichiers identiques ont été trouvés (en conservant une copie par groupe).",
        ),
        format!(
            "Environ {} pourraient être récupérés si vous supprimez les copies en double après vérification.",
            format_bytes_short(recoverable)
        ),
        recoverable,
        "Examiner les doublons",
        "/duplicates",
        Some(groups as f64),
        None,
        None,
        None,
    )]
}

fn rule_old_files(ctx: &EngineContext) -> Vec<RecommendationDto> {
    if ctx.insights.scan.is_none() || ctx.old_files_count < thresholds::OLD_FILES_MIN_COUNT {
        return Vec::new();
    }
    vec![make_rec(
        "old_files_review",
        "old_files",
        Severity::Info,
        "Fichiers anciens",
        format!(
            "{} fichier(s) indexés n'ont pas été modifiés depuis plus d'un an.",
            ctx.old_files_count
        ),
        "Cela ne signifie pas qu'ils sont inutiles — vérifiez seulement s'ils sont encore nécessaires.".into(),
        ctx.old_files_bytes,
        "Relancer une analyse",
        "/analyze",
        Some(ctx.old_files_count as f64),
        Some(thresholds::OLD_FILES_DAYS as f64),
        None,
        None,
    )]
}

fn rule_applications(ctx: &EngineContext) -> Vec<RecommendationDto> {
    if ctx.insights.scan.is_none() {
        return Vec::new();
    }
    vec![make_rec(
        "applications_review",
        "applications",
        Severity::Info,
        "Applications peu utilisées",
        "Triez les programmes installés par taille ou date d'utilisation dans Windows.".into(),
        "Giza ne désinstalle jamais automatiquement — vous gardez le contrôle à chaque étape.".into(),
        0,
        "Gérer les applications",
        "/cleanup",
        None,
        None,
        None,
        None,
    )]
}

pub fn rule_cleanup(categories: &[CleanupCategorySummary]) -> Vec<RecommendationDto> {
    let mut safe_review_bytes = 0_u64;
    let mut safe_review_files = 0_u64;
    let mut risky_bytes = 0_u64;
    for cat in categories {
        match cat.risk_level {
            RiskLevel::Safe | RiskLevel::Review => {
                safe_review_bytes = safe_review_bytes.saturating_add(cat.total_bytes);
                safe_review_files = safe_review_files.saturating_add(cat.file_count);
            }
            RiskLevel::Risky => {
                risky_bytes = risky_bytes.saturating_add(cat.total_bytes);
            }
        }
    }
    if safe_review_bytes < thresholds::CLEANUP_MIN_BYTES {
        return Vec::new();
    }
    let _ = risky_bytes;
    vec![make_rec(
        "cleanup_available",
        "cleanup",
        Severity::Info,
        "Fichiers temporaires à examiner",
        format!(
            "Environ {} répartis sur {} fichier(s) temporaires ou caches (catégories sûres ou à revue).",
            format_bytes_short(safe_review_bytes),
            safe_review_files
        ),
        "Les éléments classés à risque élevé ne sont pas inclus dans cette suggestion.".into(),
        safe_review_bytes,
        "Voir le nettoyage",
        "/cleanup",
        Some(safe_review_bytes as f64),
        Some(thresholds::CLEANUP_MIN_BYTES as f64),
        None,
        None,
    )]
}

fn compute_storage_growth(
    conn: &Connection,
    mount_path: &str,
    period_days: i32,
) -> (Option<i64>, u32) {
    let since = (Utc::now() - Duration::days(period_days as i64)).to_rfc3339();
    let Ok(rows) = storage_snapshot_repo::list_history(conn, mount_path, &since, 500) else {
        return (None, period_days as u32);
    };
    if rows.len() < 2 {
        return (None, period_days as u32);
    }
    let first = rows.first().unwrap().used_bytes;
    let last = rows.last().unwrap().used_bytes;
    (Some(last as i64 - first as i64), period_days as u32)
}

fn old_files_stats(
    conn: &Connection,
    insights: &dashboard::StorageInsights,
) -> Result<(u64, u64)> {
    let Some(scan) = &insights.scan else {
        return Ok((0, 0));
    };
    let cutoff = (Utc::now() - Duration::days(thresholds::OLD_FILES_DAYS)).to_rfc3339();
    let (count, bytes): (i64, i64) = conn.query_row(
        "SELECT COUNT(*), COALESCE(SUM(size_bytes), 0)
         FROM files
         WHERE scan_id = ?1
           AND modified_at IS NOT NULL
           AND datetime(modified_at) <= datetime(?2)",
        (scan.id, cutoff.as_str()),
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    Ok((count.max(0) as u64, bytes.max(0) as u64))
}

fn make_rec(
    id: &str,
    rec_type: &str,
    severity: Severity,
    title: &str,
    description: String,
    reason: String,
    potential_bytes: u64,
    action_label: &str,
    action_route: &str,
    detected_value: Option<f64>,
    threshold: Option<f64>,
    period_days: Option<u32>,
    mount_path: Option<String>,
) -> RecommendationDto {
    RecommendationDto {
        id: id.to_string(),
        recommendation_type: rec_type.to_string(),
        severity: severity.as_str().to_string(),
        title: title.to_string(),
        description,
        reason,
        potential_bytes,
        action_label: action_label.to_string(),
        action_route: action_route.to_string(),
        detected_value,
        threshold,
        period_days,
        mount_path,
        created_at: Utc::now().to_rfc3339(),
    }
}

fn severity_rank(severity: &str) -> u8 {
    match severity {
        "important" => 3,
        "warning" => 2,
        "info" => 1,
        _ => 0,
    }
}

fn dedupe_by_id(items: &mut Vec<RecommendationDto>) {
    let mut seen = std::collections::HashSet::new();
    items.retain(|r| seen.insert(r.id.clone()));
}

fn format_bytes_short(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.1} Go", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.0} Mo", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0} Ko", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} o")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cleanup::CleanupCategorySummary;

    fn base_ctx() -> EngineContext {
        EngineContext {
            mount_path: "C:\\".into(),
            usage_percent: 50.0,
            free_bytes: 0,
            _total_bytes: 0,
            insights: dashboard::StorageInsights {
                scan: None,
                has_duplicate_analysis: false,
                duplicate_group_count: 0,
                duplicate_recoverable_bytes: 0,
                large_files_count: 0,
                large_files_total_bytes: 0,
            },
            large_threshold_bytes: 100 * 1024 * 1024,
            growth_bytes: None,
            growth_period_days: 7,
            old_files_count: 0,
            old_files_bytes: 0,
        }
    }

    #[test]
    fn no_storage_alert_below_75_percent() {
        let mut ctx = base_ctx();
        ctx.usage_percent = 70.0;
        assert!(rule_storage_usage(&ctx).is_empty());
    }

    #[test]
    fn warning_at_85_important_at_90_and_95() {
        let mut ctx = base_ctx();
        ctx.usage_percent = 86.0;
        assert_eq!(rule_storage_usage(&ctx)[0].severity, "warning");

        ctx.usage_percent = 91.0;
        assert_eq!(rule_storage_usage(&ctx)[0].severity, "important");

        ctx.usage_percent = 96.0;
        let critical = rule_storage_usage(&ctx);
        assert_eq!(critical[0].severity, "important");
        assert!(critical[0].title.contains("saturé"));
    }

    #[test]
    fn storage_explainability_fields() {
        let mut ctx = base_ctx();
        ctx.usage_percent = 88.0;
        let rec = &rule_storage_usage(&ctx)[0];
        assert_eq!(rec.id, "storage_usage_high");
        assert_eq!(rec.detected_value, Some(88.0));
        assert_eq!(rec.threshold, Some(f64::from(thresholds::STORAGE_WARNING_PERCENT)));
    }

    #[test]
    fn large_files_none_below_min_count() {
        let mut ctx = base_ctx();
        ctx.insights.large_files_count = 2;
        assert!(rule_large_files(&ctx).is_empty());
    }

    #[test]
    fn large_files_aggregated() {
        let mut ctx = base_ctx();
        ctx.insights.scan = Some(crate::large_files::ScanSummaryDto {
            id: 1,
            root_path: "C:\\x".into(),
            files_seen: 10,
            bytes_total: 0,
            finished_at: None,
        });
        ctx.insights.large_files_count = 5;
        ctx.insights.large_files_total_bytes = 10 * 1024 * 1024 * 1024;
        ctx.large_threshold_bytes = 2 * 1024 * 1024 * 1024;
        let recs = rule_large_files(&ctx);
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].id, "large_files_detected");
        assert!(recs[0].action_label.contains("Examiner"));
    }

    #[test]
    fn duplicates_none_and_group_recoverable() {
        let mut ctx = base_ctx();
        assert!(rule_duplicates(&ctx).is_empty());

        ctx.insights.scan = Some(crate::large_files::ScanSummaryDto {
            id: 1,
            root_path: "C:\\x".into(),
            files_seen: 2,
            bytes_total: 0,
            finished_at: None,
        });
        ctx.insights.has_duplicate_analysis = true;
        ctx.insights.duplicate_group_count = 1;
        ctx.insights.duplicate_recoverable_bytes = 5 * 1024 * 1024 * 1024;
        let recs = rule_duplicates(&ctx);
        assert_eq!(recs[0].potential_bytes, 5 * 1024 * 1024 * 1024);
    }

    #[test]
    fn cleanup_safe_review_only_not_risky() {
        let cats = vec![
            CleanupCategorySummary {
                category: "user_temp".into(),
                label: "Temp".into(),
                file_count: 10,
                total_bytes: 500 * 1024 * 1024,
                risk_level: RiskLevel::Safe,
            },
            CleanupCategorySummary {
                category: "system".into(),
                label: "Système".into(),
                file_count: 5,
                total_bytes: 10 * 1024 * 1024 * 1024,
                risk_level: RiskLevel::Risky,
            },
        ];
        let recs = rule_cleanup(&cats);
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].potential_bytes, 500 * 1024 * 1024);
    }

    #[test]
    fn cleanup_below_threshold_no_reco() {
        let cats = vec![CleanupCategorySummary {
            category: "user_temp".into(),
            label: "Temp".into(),
            file_count: 1,
            total_bytes: 1024,
            risk_level: RiskLevel::Safe,
        }];
        assert!(rule_cleanup(&cats).is_empty());
    }

    #[test]
    fn growth_none_or_low() {
        let mut ctx = base_ctx();
        assert!(rule_storage_growth(&ctx).is_empty());
        ctx.growth_bytes = Some(1024);
        ctx.growth_period_days = 7;
        assert!(rule_storage_growth(&ctx).is_empty());
    }

    #[test]
    fn growth_significant() {
        let mut ctx = base_ctx();
        ctx.growth_bytes = Some(8 * 1024 * 1024 * 1024);
        ctx.growth_period_days = 7;
        let recs = rule_storage_growth(&ctx);
        assert_eq!(recs[0].id, "storage_growth");
        assert_eq!(recs[0].period_days, Some(7));
    }

    #[test]
    fn max_five_and_order() {
        let mut ctx = base_ctx();
        ctx.usage_percent = 93.0;
        ctx.growth_bytes = Some(10 * 1024 * 1024 * 1024);
        ctx.insights.scan = Some(crate::large_files::ScanSummaryDto {
            id: 1,
            root_path: "C:\\x".into(),
            files_seen: 100,
            bytes_total: 0,
            finished_at: None,
        });
        ctx.insights.has_duplicate_analysis = true;
        ctx.insights.duplicate_group_count = 20;
        ctx.insights.duplicate_recoverable_bytes = 50 * 1024 * 1024 * 1024;
        ctx.insights.large_files_count = 50;
        ctx.insights.large_files_total_bytes = 80 * 1024 * 1024 * 1024;
        ctx.old_files_count = 100;
        ctx.old_files_bytes = 1024;

        let cats = vec![CleanupCategorySummary {
            category: "user_temp".into(),
            label: "Temp".into(),
            file_count: 20,
            total_bytes: 2 * 1024 * 1024 * 1024,
            risk_level: RiskLevel::Safe,
        }];
        let recs = finalize_recommendations(&ctx, Some(&cats));
        assert!(recs.len() <= MAX_RECOMMENDATIONS);
        assert_eq!(recs[0].severity, "important");
        let ids: Vec<_> = recs.iter().map(|r| r.id.clone()).collect();
        let ids2: Vec<_> = finalize_recommendations(&ctx, Some(&cats))
            .iter()
            .map(|r| r.id.clone())
            .collect();
        assert_eq!(ids, ids2);
    }

    #[test]
    fn dedupe_by_stable_id() {
        let mut ctx = base_ctx();
        ctx.usage_percent = 90.0;
        let mut items = rule_storage_usage(&ctx);
        items.extend(rule_storage_usage(&ctx));
        dedupe_by_id(&mut items);
        assert_eq!(items.len(), 1);
    }
}
