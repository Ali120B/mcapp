import { useEffect, useState } from 'react';
import { searchProjects, type ModrinthProject } from '../hooks/api';
import { EmptyState, OfflineBanner, useToast } from '../components';
const TABS=['modpack','mod','resourcepack','datapack','shader'];
export function DiscoverPage(){const [query,setQuery]=useState('');const [tab,setTab]=useState('modpack');const [items,setItems]=useState<ModrinthProject[]>([]);const [page,setPage]=useState(0); const [offline,setOffline]=useState(false); const {push}=useToast();
async function load(){
  try { const r=await searchProjects({query,projectType:tab,limit:20,offset:page*20,sort:'downloads'});setItems(r.hits); setOffline(false); localStorage.setItem(`discover:${tab}:${query}:${page}`, JSON.stringify(r.hits)); }
  catch { const cached = localStorage.getItem(`discover:${tab}:${query}:${page}`); if(cached){ setItems(JSON.parse(cached) as ModrinthProject[]); setOffline(true); push('info','API unreachable, loaded cached discover results.'); } else { setItems([]); setOffline(true); push('error','Discover is offline and no cache is available.'); } }
}
useEffect(()=>{void load()},[tab,page]);
return <section><h2>Discover</h2><OfflineBanner offline={offline}/><div className='row'>{TABS.map(t=><button key={t} onClick={()=>{setTab(t);setPage(0)}}>{t}</button>)}</div><div className='toolbar'><input value={query} onChange={e=>setQuery(e.target.value)} /><button onClick={()=>{setPage(0);void load();}}>Search</button></div>{items.length?<ul className='cards'>{items.map(i=><li key={i.project_id}><b>{i.title}</b><p>{i.description}</p></li>)}</ul>:<EmptyState title='No results yet. Try another query or category.'/>}<div className='row'><button onClick={()=>setPage(Math.max(0,page-1))}>Prev</button><span>Page {page+1}</span><button onClick={()=>setPage(page+1)}>Next</button></div></section>}
