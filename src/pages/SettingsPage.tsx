import { useEffect, useState } from 'react';
import { detectJavaInstallations, type JavaInstallation } from '../hooks/api';

export type AppSettings = {
  appearance: 'dark' | 'light' | 'oled' | 'system';
  blur_effects: boolean;
  font_size: number;
  language: string;
  telemetry_enabled: boolean;
  crash_reports: boolean;
  default_loader: string;
  default_game_version: string;
  default_ram_mb: number;
  download_concurrency: number;
  cache_size_mb: number;
};

const defaults: AppSettings = {
  appearance: 'dark', blur_effects: true, font_size: 14, language: 'English', telemetry_enabled: false,
  crash_reports: false, default_loader: 'fabric', default_game_version: '1.21.1', default_ram_mb: 4096,
  download_concurrency: 4, cache_size_mb: 2048,
};

export function SettingsPage(){
  const [settings, setSettings] = useState<AppSettings>(defaults);
  const [javas, setJavas] = useState<JavaInstallation[]>([]);

  useEffect(()=>{
    const raw = localStorage.getItem('mcapp.settings');
    if(raw) setSettings({...defaults, ...JSON.parse(raw) as AppSettings});
    void detectJavaInstallations().then(setJavas);
  },[]);

  const save = (next: AppSettings) => { setSettings(next); localStorage.setItem('mcapp.settings', JSON.stringify(next)); };

  return <section>
    <h2>Application Settings</h2>
    <h3>Appearance</h3>
    <div className='toolbar'>
      {['dark','light','oled','system'].map(a=><button key={a} className={settings.appearance===a?'active-pill':''} onClick={()=>save({...settings,appearance:a as AppSettings['appearance']})}>{a}</button>)}
      <label><input type='checkbox' checked={settings.blur_effects} onChange={e=>save({...settings,blur_effects:e.target.checked})}/> Blur</label>
      <label>Font <input type='number' value={settings.font_size} onChange={e=>save({...settings,font_size:Number(e.target.value)})}/></label>
    </div>
    <h3>Privacy</h3>
    <div className='toolbar'>
      <label><input type='checkbox' checked={settings.telemetry_enabled} onChange={e=>save({...settings,telemetry_enabled:e.target.checked})}/> Telemetry</label>
      <label><input type='checkbox' checked={settings.crash_reports} onChange={e=>save({...settings,crash_reports:e.target.checked})}/> Crash reports</label>
    </div>
    <h3>Java installations</h3>
    <ul className='cards'>{javas.map((j,idx)=><li key={idx}><b>Java {j.major}</b><p>{j.version}</p><small>{j.path}</small></li>)}</ul>
    <h3>Default instance options</h3>
    <div className='toolbar'>
      <input value={settings.default_loader} onChange={e=>save({...settings,default_loader:e.target.value})}/>
      <input value={settings.default_game_version} onChange={e=>save({...settings,default_game_version:e.target.value})}/>
      <input type='number' value={settings.default_ram_mb} onChange={e=>save({...settings,default_ram_mb:Number(e.target.value)})}/>
    </div>
    <h3>Resource management</h3>
    <div className='toolbar'>
      <label>Concurrency <input type='number' value={settings.download_concurrency} onChange={e=>save({...settings,download_concurrency:Number(e.target.value)})}/></label>
      <label>Cache MB <input type='number' value={settings.cache_size_mb} onChange={e=>save({...settings,cache_size_mb:Number(e.target.value)})}/></label>
      <button onClick={()=>save({...settings,cache_size_mb:0})}>Clear cache</button>
    </div>
  </section>;
}
