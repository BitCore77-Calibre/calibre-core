const stages = [
  ["Genesis", "Network foundations", "Genesis allocation and chain identity", "chain_getBlockHash(0)"],
  ["Node startup", "Network foundations", "One connection and health record per configured node", "system_health"],
  ["Peer network", "Network foundations", "Observed peer counts; topology requires node instrumentation", "system_health"],
  ["Wallets & QUTXOs", "Network foundations", "Wallet and QUTXO adapter requires CALIBRE runtime source", "not connected"],
  ["Build transaction", "Transaction enters network", "Transaction construction requires wallet integration", "not connected"],
  ["Signing", "Transaction enters network", "Signature type and verification require runtime integration", "not connected"],
  ["Submit & broadcast", "Transaction enters network", "Submission tracing requires a signed transaction runner", "not connected"],
  ["7-node mempool", "Transaction enters network", "Per-node pending extrinsic snapshots where RPC permits", "author_pendingExtrinsics"],
  ["Validity & conflicts", "Consensus and execution", "Runtime validation evidence requires transaction instrumentation", "not instrumented"],
  ["Slots & epochs", "Consensus and execution", "Session, BABE and GRANDPA authorities from finalized node state; Phase 9.4 lab result when configured", "finalized state RPC"],
  ["Block production", "Consensus and execution", "Observed chain head; author identity needs decoded consensus digest", "chain_getHeader"],
  ["Block execution", "Consensus and execution", "Extrinsics available; dispatch events need SCALE metadata decoding", "chain_getBlock"],
  ["Block propagation", "Settlement and search", "Compare heights and finalized hashes across independent nodes", "chain_getHeader"],
  ["Finality", "Settlement and search", "Independently observed finalized heads on each node", "chain_getFinalizedHead"],
  ["State & economy", "Settlement and search", "Symbol and decimals available; supply and fees need runtime storage", "system_properties"],
  ["Explorer indexing", "Settlement and search", "Recent block observations stored; full historical indexing pending", "chain_getBlockHash"],
  ["Live overview", "Research and audit", "One view of connected nodes and current chain heads", "observed RPC"],
  ["Experiment control", "Research and audit", "Wallet tests require current chain and signed transaction adapter", "not connected"],
  ["Audit & incidents", "Research and audit", "Node health, genesis mismatch, and collector errors retained", "local evidence log"],
  ["Replay & reports", "Research and audit", "Recorded observations, with source and integrity digests", "local evidence log"],
];
const groups = [
  "A · Network foundations", "B · Transaction enters network", "C · Consensus and execution",
  "D · Settlement and search", "E · Research and audit"
];
const state = {snapshot:null,events:[],selected:null, block:null};
const $ = id => document.getElementById(id);
const short = (s,n=12) => s ? `${String(s).slice(0,n)}…` : "—";
const num = v => Number.isInteger(v) ? v.toLocaleString() : "—";
const online = () => state.snapshot?.nodes?.filter(n=>n.online) || [];
const net = () => state.snapshot?.network || {};
const incidents = () => state.events.filter(e=>["invariant_failure","collector_error","node_offline"].includes(e.category));

function tileData(index) {
  const nodes=online(), n=net(), first=nodes[0], any=nodes.length>0;
  const phase94=state.snapshot?.phase94;
  const pool=nodes.map(x=>x.pool).filter(x=>x?.state==="snapshot");
  const pcount=pool.reduce((sum,x)=>sum+x.count,0);
  const near=(key)=>nodes.map(x=>x[key]).filter(x=>x!==undefined&&x!==null);
  const values={
    1:[any?short(first.genesis_hash,16):"Awaiting node",any?`${first.chain} · allocation not verified`:"No genesis RPC",any?(n.genesis_match?"OBSERVED":"MISMATCH"):"OFFLINE"],
    2:[`${nodes.length} / ${n.configured||1} nodes`,any?`${nodes.map(x=>x.id).join(" · ")}`:"Configured RPC endpoints not reachable",any?"OBSERVED":"OFFLINE"],
    3:[any?`${near("health").reduce((v,h)=>v+(h.peers||0),0)} peer observations`:"Awaiting peers","Peer topology needs node instrumentation",any?"OBSERVED":"OFFLINE"],
    4:["Not connected","Wallet balances and QUTXOs require runtime adapter","BLOCKED"],
    5:["Not connected","No transaction builder without current CALIBRE code","BLOCKED"],
    6:["Not connected","No signature evidence without wallet and runtime","BLOCKED"],
    7:["Not connected","No signed submission has been made here","BLOCKED"],
    8:[pool.length?`${pcount} pending observations`:"Pool RPC unavailable",`${pool.length} / ${nodes.length} online nodes expose a pool snapshot`,pool.length?"OBSERVED":"UNAVAILABLE"],
    9:["Not instrumented","No per-transaction validation events connected","BLOCKED"],
    10:[phase94?`Session ${Math.min(...(phase94.sessions||[0]))} · ${phase94.status}`:"Not configured",
      phase94?(phase94.reason||`${phase94.authority_count||"—"} active authorities · 600-slot epoch`):"Start the Phase 9.4 local lab to observe the election",
      phase94?.status||"UNAVAILABLE"],
    11:[any?`Block ${num(n.best_height)}`:"Awaiting blocks",any?`Best ${short(first.best_hash)}`:"Node offline",any?"OBSERVED":"OFFLINE"],
    12:["Events not decoded","Runtime event decoding needs matching chain metadata","PARTIAL"],
    13:[any?`${new Set(near("best_hash")).size} local best head(s)`:"Awaiting nodes",any?`${nodes.length} nodes independently observed`:"No node observations",any?"OBSERVED":"OFFLINE"],
    14:[any?`Finalized ${num(n.finalized_height)}`:"Awaiting finality",any?`${new Set(near("finalized_hash")).size} head hash(es), heights may differ`:"No finality observations",any?"OBSERVED":"OFFLINE"],
    15:[any?(first.properties?.tokenSymbol||"Symbol unknown"):"Awaiting properties","Issuance and fee destinations not yet verified",any?"PARTIAL":"OFFLINE"],
    16:[any?short(first.best_hash,20):"Awaiting block",any?"Live best hash observed · backfill pending":"No chain index yet",any?"PARTIAL":"OFFLINE"],
    17:[any?`${nodes.length} connected node(s)`:"No live chain",n.genesis_match===false?"GENESIS MISMATCH":"Only observed data is displayed",any?"OBSERVED":"OFFLINE"],
    18:["Runs disabled","Signed workload runner needs calibre-template","BLOCKED"],
    19:[`${incidents().length} incident record(s)`,"Health changes · genesis mismatch · collector errors","RECORDED"],
    20:[`${state.events.length} recent records`,"Hash-linked local observations · no transaction replay yet","RECORDED"]
  };
  return values[index];
}

function renderBoard() {
  const board=$("board");board.replaceChildren();
  for(let group=0;group<5;group++){
    const band=document.createElement("section");band.className="band";
    const name=document.createElement("div");name.className="band-title";
    const title=document.createElement("b");title.textContent=groups[group];
    const range=document.createElement("span");range.textContent=`${String(group*4+1).padStart(2,'0')} → ${String(group*4+4).padStart(2,'0')}`;
    name.append(title,range);band.append(name);
    const row=document.createElement("div");row.className="band-row";
    for(let offset=0;offset<4;offset++){
      const i=group*4+offset+1,stage=stages[i-1],d=tileData(i);
      const b=document.createElement("button");b.type="button";b.className="tile";
      if(d[2]==="OFFLINE"||d[2]==="BLOCKED")b.classList.add("off");
      if(i===19&&incidents().length||i===1&&net().genesis_match===false||d[2]==="FAIL")b.classList.add("issue");
      if(d[2]==="OBSERVED"||d[2]==="PASS")b.classList.add("ready");
      const top=document.createElement("div");top.className="tile-head";
      const number=document.createElement("span");number.className="number";number.textContent=String(i).padStart(2,"0");
      const heading=document.createElement("span");heading.className="tile-title";heading.textContent=stage[0];
      const badge=document.createElement("span");badge.className="tile-state";badge.textContent=d[2];
      if(d[2]==="OBSERVED"||d[2]==="PASS")badge.classList.add("live");if(d[2]==="MISMATCH"||d[2]==="FAIL")badge.classList.add("issue");
      top.append(number,heading,badge);
      const metric=document.createElement("div");metric.className="tile-main";metric.textContent=d[0];
      const sub=document.createElement("div");sub.className="tile-sub";sub.textContent=d[1];
      const source=document.createElement("div");source.className="tile-source";
      const left=document.createElement("span");left.textContent=stage[3];
      const right=document.createElement("em");right.textContent="Inspect ↗";
      source.append(left,right);b.append(top,metric,sub,source);
      b.addEventListener("click",()=>openDetail(i));row.append(b);
    }
    band.append(row);board.append(band);
  }
}

function renderHeader(){
  const n=net(),first=online()[0];
  $("network-status").textContent=n.status||"OFFLINE";
  $("network-status").className=`status ${n.status==="LIVE"?"live":"offline"}`;
  $("nodes-count").textContent=`${n.online||0} / ${n.configured||1}`;
  $("best-height").textContent=num(n.best_height);
  $("finalized-height").textContent=num(n.finalized_height);
  $("genesis-id").textContent=short(first?.genesis_hash,12);
  $("last-updated").textContent=state.snapshot?.collected_at||"Not connected";
  const ts=state.snapshot?.collected_at?Math.round((Date.now()-Date.parse(state.snapshot.collected_at))/1000):null;
  $("freshness").textContent=ts===null?"Awaiting first observation":`${Math.max(0,ts)}s since observation`;
  const phase94=state.snapshot?.phase94;
  $("notice").textContent=n.status==="LIVE"?
    (n.genesis_match===false?"ALERT: connected RPC nodes do not match the expected genesis. Inspect Audit & Incidents.":
      phase94?`Phase 9.4 local lab: ${phase94.status}. Open Slots & epochs for the observed authority and finality evidence.`:
        "Live local RPC observations. Transaction signing and decoded events await repository integration."):
    "No CALIBRE RPC node is connected. Start your existing chain separately; this monitor never invents blocks or transactions.";
  $("notice").className=n.genesis_match===false?"notice alert":"notice";
}

function addTable(parent,rows){
  const table=document.createElement("table");
  for(const [k,v] of rows){const tr=document.createElement("tr");const key=document.createElement("td");key.textContent=k;
    const val=document.createElement("td");val.textContent=typeof v==="object"?JSON.stringify(v):String(v??"—");tr.append(key,val);table.append(tr)}
  parent.append(table);
}
function addPre(parent,value){const pre=document.createElement("pre");pre.textContent=JSON.stringify(value,null,2);parent.append(pre)}

async function openDetail(i){
  state.selected=i;state.block=null;
  $("detail-number").textContent=`STAGE ${String(i).padStart(2,"0")} / 20`;
  $("detail-title").textContent=stages[i-1][0];
  $("detail-description").textContent=stages[i-1][2];
  $("dialog").hidden=false;
  $("close-detail").focus();
  renderDetail();
  if([11,12,16].includes(i)){
    const node=online()[0];
    if(node?.best_hash){
      try{const response=await fetch(`/api/block/${node.best_hash}`);
        state.block=await response.json();
      }catch(error){state.block={error:String(error)}}
      if(state.selected===i)renderDetail();
    }
  }
}

function renderDetail(){
  const i=state.selected;if(!i)return;
  const container=$("detail-evidence");container.replaceChildren();
  const nodes=state.snapshot?.nodes||[],n=net(),first=online()[0];
  if(i===2||i===3||i===8||i===13||i===14){
    addTable(container,[["Node","Health, height, finalized, peers, pool"]]);
    for(const node of nodes){addTable(container,[[node.id,{
      endpoint:node.url,online:node.online,genesis_hash:node.genesis_hash,
      best_height:node.best_height,best_hash:node.best_hash,finalized_height:node.finalized_height,
      finalized_hash:node.finalized_hash,peers:node.health?.peers,pool:node.pool,
      errors:node.errors
    }]])}
  }else if(i===1){
    addTable(container,[["Network",first?.chain||"No connected node"],["Genesis hash",first?.genesis_hash],
      ["All observed hashes",n.genesis_hashes||[]],["Hash agreement",n.genesis_match],
      ["Issuance/allocation","UNVERIFIED — chain specification is not present"]]);
  }else if(i===15){addTable(container,[["Chain properties",first?.properties||"No connected node"],
    ["Fee destinations","UNVERIFIED — runtime integration required"],
    ["777 million CAL genesis","UNVERIFIED — no chain specification available"]]);
  }else if([11,12,16].includes(i)){
    if(state.block)addPre(container,state.block);
    else addTable(container,[["Latest observed block hash",first?.best_hash||"No connected node"],
      ["Raw block","Loading node RPC record; runtime events not decoded"]]);
  }else if(i===10){
    const phase94=state.snapshot?.phase94;
    if(phase94){
      addTable(container,[["Result",phase94.status],["Explanation",phase94.reason||"Awaiting node evidence"],
        ["Observed sessions",phase94.sessions||[]],["Common finalized height",phase94.common_finalized_height],
        ["Common finalized hash",phase94.common_finalized_hash],["Hash agreement",phase94.common_hash_agreement],
        ["Lab scope",phase94.scope],["Limits",phase94.limits]]);
      for(const node of nodes)addTable(container,[[node.id,{online:node.online,finalized_height:node.finalized_height,
        session:node.phase94?.session,epoch_slots:node.phase94?.epoch_slots,
        spec_version:node.phase94?.spec_version,validators:node.phase94?.validators,
        babe:node.phase94?.babe,grandpa:node.phase94?.grandpa,error:node.phase94?.error}]]);
    }else addTable(container,[["Status","Phase 9.4 local lab is not configured"]]);
  }else if(i===19||i===20){addTable(container,[["Evidence records",state.events.length],
    ["Integrity","Entries include chained SHA-256 digests; no external anchoring"],
    ["Classification","Connection failures and genesis mismatches only at this stage"]]);
  }else if(i===17){addPre(container,{network:n,nodes:nodes.map(x=>({id:x.id,online:x.online,
    best_height:x.best_height,finalized_height:x.finalized_height,errors:x.errors}))});
  }else addTable(container,[["Connection status","Backend feature not connected"],
    ["What is needed",stages[i-1][2]],["Node evidence",first?"At least one RPC node is responding":"No node is connected"]]);
  const list=$("detail-events");list.replaceChildren();
  const related=(i===19?state.events.filter(e=>["invariant_failure","collector_error","node_offline"].includes(e.category)):
    i===20?state.events:state.events.filter(e=>i===2||i===3?e.category.startsWith("node_"):
      i===11||i===13||i===16?e.category==="best_head":i===14?e.category==="finalized_head":
      i===1?e.category==="invariant_failure":i===10?e.category.startsWith("phase94_"):false)).slice(0,20);
  if(!related.length){const empty=document.createElement("p");empty.textContent="No matching observation recorded yet.";list.append(empty)}
  for(const e of related){const row=document.createElement("div");row.className="event";
    const label=document.createElement("strong");label.textContent=`${e.category} · ${e.node||"network"}`;
    const when=document.createElement("time");when.textContent=e.observed_at;
    const info=document.createElement("div");info.textContent=`${e.description} · ${e.reference||"no hash"}`;
    row.append(label,when,info);list.append(row)}
}

let loading=false;
async function load(){
  if(loading)return;loading=true;
  try{const [snapshot,events]=await Promise.all([
    fetch("/api/snapshot",{cache:"no-store"}),fetch("/api/events?limit=100",{cache:"no-store"})]);
    if(!snapshot.ok||!events.ok)throw new Error(`Collector returned ${snapshot.status}/${events.status}`);
    state.snapshot=await snapshot.json();state.events=(await events.json()).events||[];
    renderHeader();renderBoard();if(state.selected)renderDetail();
  }catch(error){$("network-status").textContent="MONITOR ERROR";$("network-status").className="status offline";
    $("notice").textContent=`Observatory backend unavailable: ${String(error)}`;$("notice").className="notice alert";
  }finally{loading=false}
}
$("refresh").addEventListener("click",load);
$("close-detail").addEventListener("click",()=>{$("dialog").hidden=true;state.selected=null});
document.querySelector(".shade").addEventListener("click",()=>{$("dialog").hidden=true;state.selected=null});
document.addEventListener("keydown",event=>{if(event.key==="Escape"&&!$("dialog").hidden){$("dialog").hidden=true;state.selected=null}});
renderBoard();load();setInterval(load,3000);
