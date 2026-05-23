import React, { createContext, useContext, useEffect, useState, type ReactNode } from 'react';
import { useNavigate } from 'react-router-dom';
import { listen } from '@tauri-apps/api/event';

type Toast = { id: number; kind: 'success'|'error'|'info'; message: string };
const ToastCtx = createContext<{push:(kind:Toast['kind'],message:string)=>void}>({push:()=>{}});
export function useToast(){ return useContext(ToastCtx); }

export function AppProviders({children}:{children:ReactNode}){
  const [toasts,setToasts]=useState<Toast[]>([]);
  const [downloads,setDownloads]=useState<Record<string,{step:string;progress:number}>>({});
  const push=(kind:Toast['kind'],message:string)=>{ const id=Date.now()+Math.random(); setToasts(t=>[...t,{id,kind,message}]); setTimeout(()=>setToasts(t=>t.filter(x=>x.id!==id)),3000); };

  useEffect(()=>{ let unlisten: undefined | (()=>void); void listen<any>('install:progress', (event)=>{
    const p = event.payload as { instance_id:string; step:string; progress:number };
    setDownloads(d=>({ ...d, [p.instance_id]: { step: p.step, progress: p.progress }}));
    if (p.progress >= 1) push('success', `Install complete for ${p.instance_id}`);
  }).then(fn=>{ unlisten=fn; }); return ()=>{ if(unlisten) unlisten();}; },[]);

  return <ToastCtx.Provider value={{push}}>{children}
    <div className='toast-stack'>{toasts.map(t=><div key={t.id} className={`toast ${t.kind}`}>{t.message}</div>)}</div>
    <div className='download-tray'>
      <h4>Downloads</h4>
      {!Object.keys(downloads).length && <small>No active downloads</small>}
      {Object.entries(downloads).map(([id,d])=><div key={id}><b>{id}</b><small>{d.step}</small><progress max={1} value={Math.max(0,Math.min(1,d.progress))}/></div>)}
    </div>
  </ToastCtx.Provider>;
}

export function EmptyState({title,action}:{title:string;action?:ReactNode}){ return <div className='empty'><p>{title}</p>{action}</div>; }

export class PageErrorBoundary extends React.Component<{children:ReactNode},{hasError:boolean}>{
  constructor(props:{children:ReactNode}){ super(props); this.state={hasError:false}; }
  static getDerivedStateFromError(){ return {hasError:true}; }
  componentDidCatch(err:unknown){ console.error(err); }
  render(){ if(this.state.hasError) return <EmptyState title='This section crashed. Please reload.' />; return this.props.children; }
}

export function GlobalShortcuts(){
  const nav = useNavigate();
  useEffect(()=>{ const onKey=(e:KeyboardEvent)=>{ if((e.ctrlKey||e.metaKey)&&e.key===','){ e.preventDefault(); nav('/settings'); }
    if((e.ctrlKey||e.metaKey)&&e.key.toLowerCase()==='n'){ e.preventDefault(); nav('/library'); }
  }; window.addEventListener('keydown',onKey); return ()=>window.removeEventListener('keydown',onKey); },[nav]);
  return null;
}

export function OfflineBanner({offline}:{offline:boolean}){ if(!offline) return null; return <div className='offline-banner'>Offline — showing cached results</div>; }
