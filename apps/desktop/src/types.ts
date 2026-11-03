export type Session = {id:string;start_ms:number;end_ms:number;app:string;idle:boolean;domain:string|null;profile_id?:string|null;category:string};
export type RuleConfig = {action:string;group_domains:string[];group_apps?:string[];group_categories?:string[];excluded_domains?:string[];excluded_apps?:string[];streak_gap_seconds?:number|null;qualified_plays:number|null;streak_plays?:number|null;streak_window_minutes?:number|null;include_replays:boolean;weekdays:number[];start_time:string|null;end_time:string|null};
export type Rule = {id:string;name:string;domain:string;path_prefix:string|null;expires_at_ms:number|null;budget_minutes:number|null;enabled:boolean;enforced:boolean;config?:RuleConfig;effective_until_ms?:number|null};
export type Goal = {id:string;name:string;target_minutes:number;apps:string[];domains:string[];enabled:boolean;created_ms:number;observed_ms_today?:number};
export type Focus = {id:string;goal_id:string|null;start_ms:number;end_ms:number;difficulty:string;block_domains:string[]};
export type Video = {platform:string;video_id:string;instances:number;qualified_plays:number;watched_ms:number;last_seen_ms:number};
export type Insights = {categories:Record<string,number>;browser_comparison:Record<string,number>;hourly_distribution_ms:number[];week:{current_ms:number;previous_ms:number;delta_ms:number;caution:string};attention_switches_this_week:number;shorts_longest_streak_90d:number;days_observed_90d:number;prediction:{available:boolean;reason?:string;hour_local?:number;probability?:number;interval_low?:number;interval_high?:number;confidence?:string;observed_days?:number;hour_samples?:number;signal?:string}};
export type PrivacySettings = {retention_days:number;idle_threshold_ms:number;excluded_apps:string[];excluded_domains:string[];video_history_visible:boolean;distraction_categories:string[];prediction_opt_in:boolean};
export type Day = {date:string;active_ms:number;development_ms:number;entertainment_ms:number;observed:boolean};
export type Snapshot = {
  ok:boolean;schema_version:string;observed_at_ms:number;tracking_paused:boolean;platform_supported:boolean;browser_connected:boolean;browser_rules_acknowledged?:boolean;browser_rules_count?:number;
  metrics:{active_ms:number;development_ms:number;entertainment_ms:number;distraction_ms?:number|null;qualified_plays:number;unique_videos:number;replays:number;playback_ms:number;watch_qualified_plays?:number;watch_unique_videos?:number;watch_playback_ms?:number};
  current:Session|null;sessions:Session[];heatmap:Day[];rules:Rule[];goals?:Goal[];focus?:Focus|null;focus_goal_hint?:{kind:string;text:string;since_ms:number}|null;
  coverage:{windows_collector:string;browser:string;no_data_is_not_zero:boolean};
};
export type Page = 'Overview'|'Live Activity'|'Activity History'|'Video Intelligence'|'Focus & Goals'|'Blocking Center'|'Automations'|'Behavior Insights'|'AI Coach'|'Devices'|'Settings & Privacy';
export const pages:Page[]=['Overview','Live Activity','Activity History','Video Intelligence','Focus & Goals','Blocking Center','Automations','Behavior Insights','AI Coach','Devices','Settings & Privacy'];
