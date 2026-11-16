import {Icon} from './icons';
import {BrowserSetupWizard} from './BrowserSetupWizard';
import {BrowserProfileConnections} from './TrackingDoctor';
import {PrivacyPanel, StartupPanel} from './Advanced';
import {CategorySettings} from './CategorySettings';
import {PrivacySessionPanel} from './PrivacySessionPanel';
import {BackupPanel, UpdatePanel} from './ReleaseFeatures';
import type {Snapshot} from './types';

const sections = [
  ['general','General','settings'],
  ['browser','Browser','branch'],
  ['privacy','Privacy','shield'],
  ['labels','Labels & distractions','target'],
  ['backups','Backups & updates','download'],
] as const;
export type SettingsSection = typeof sections[number][0];

type Props = {
  section:SettingsSection;
  onSection:(section:SettingsSection)=>void;
  snapshot:Snapshot|null;
  readonly:boolean;
  preview:boolean;
  lite:boolean;
  presentationPrivacy:boolean;
  onPause:()=>void;
  onLite:(enabled:boolean)=>void;
  onPresentationPrivacy:(enabled:boolean)=>void;
  onUpdate:()=>void;
  onExport:()=>void;
  onDelete:()=>void;
  exportBusy:boolean;
};

export function SettingsWorkspace({
  section, onSection, snapshot, readonly, preview, lite, presentationPrivacy,
  onPause, onLite, onPresentationPrivacy, onUpdate, onExport, onDelete, exportBusy,
}:Props) {
  const paused = snapshot?.tracking_paused ?? false;
  return <div className="settings-workspace">
    <nav className="settings-tabs" aria-label="Settings sections">
      {sections.map(([id,label,icon]) => <button type="button" key={id} aria-pressed={section === id}
        onClick={() => onSection(id)}><Icon name={icon} size={16}/><span>{label}</span></button>)}
    </nav>
    <div className="settings-card-stack" aria-label={sections.find(([id]) => id === section)?.[1]}>
      {section === 'general' && <>
        <section className="surface padded settings-card">
          <h3>App preferences</h3>
          <div className="settings-row"><div><strong>Windows foreground tracking</strong><small>Record time by application name.</small></div>
            <button type="button" disabled={readonly} className={'toggle' + (paused ? '' : ' active')}
              aria-label="Toggle tracking" aria-pressed={!paused} onClick={onPause}><span/></button></div>
          <div className="settings-row"><div><strong>Lite Mode</strong><small>Reduce animations and background refreshes.</small></div>
            <button type="button" className={'toggle' + (lite ? ' active' : '')}
              aria-label="Toggle Lite Mode" aria-pressed={lite} onClick={() => onLite(!lite)}><span/></button></div>
          <div className="settings-row"><div><strong>Presentation Mode</strong><small>Hide activity details while sharing your screen.</small></div>
            <button type="button" className={'toggle' + (presentationPrivacy ? ' active' : '')}
              aria-label="Toggle Presentation Mode" aria-pressed={presentationPrivacy}
              onClick={() => onPresentationPrivacy(!presentationPrivacy)}><span/></button></div>
        </section>
        <StartupPanel/>
      </>}
      {section === 'browser' && <>
        <BrowserSetupWizard compact/>
        <BrowserProfileConnections readonly={readonly}/>
      </>}
      {section === 'privacy' && <>
        <PrivacyPanel readonly={readonly} refreshKey={0} onUpdate={onUpdate}/>
        <PrivacySessionPanel readonly={readonly} onUpdate={onUpdate}/>
        <section className="surface padded settings-card">
          <h3>Activity history</h3>
          <div className="settings-row"><div><strong>Delete tracked history</strong><small>Remove locally recorded activity and video history.</small></div>
            <button type="button" className="danger-btn" disabled={readonly} onClick={onDelete}>Delete history</button></div>
        </section>
      </>}
      {section === 'labels' && <CategorySettings readonly={readonly} sessions={snapshot?.sessions ?? []} onUpdate={onUpdate}/>}
      {section === 'backups' && <>
        <BackupPanel disabled={readonly}/>
        <UpdatePanel disabled={readonly}/>
        <section className="surface padded settings-card">
          <div className="panel-header"><div><h3>Export local activity</h3><p>Download an activity snapshot for review.</p></div>
            <button type="button" className="secondary-btn" disabled={readonly || exportBusy} onClick={onExport}>
              <Icon name="download" size={16}/> {exportBusy ? 'Exporting…' : 'Export complete local snapshot'}
            </button>
          </div>
        </section>
        {readonly && !preview && <p className="form-hint">Connect to the local agent to manage backups and updates.</p>}
      </>}
    </div>
  </div>;
}
