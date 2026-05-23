import { useEffect, useState } from 'react';
import { searchProjects, type ModrinthProject } from '../hooks/api';
const TABS=['modpack','mod','resourcepack','datapack','shader'];
export function DiscoverPage(){const [query,setQuery]=useState('');const [tab,setTab]=useState('modpack');const [items,setItems]=useState<ModrinthProject[]>([]);const [page,setPage]=useState(0);
async function load(){const r=await searchProjects({query,projectType:tab,limit:20,offset:page*20,sort:'downloads'});setItems(r.hits)}
useEffect(()=>{void load()},[tab,page]);
return <section><h2>Discover</h2><div className='row'>{TABS.map(t=><button key={t} onClick={()=>{setTab(t);setPage(0)}}>{t}</button>)}</div><div className='toolbar'><input value={query} onChange={e=>setQuery(e.target.value)} /><button onClick={()=>{setPage(0);void load();}}>Search</button></div><ul className='cards'>{items.map(i=><li key={i.project_id}><b>{i.title}</b><p>{i.description}</p></li>)}</ul><div className='row'><button onClick={()=>setPage(Math.max(0,page-1))}>Prev</button><span>Page {page+1}</span><button onClick={()=>setPage(page+1)}>Next</button></div></section>}
