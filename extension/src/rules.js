// Pure helper duplicated in unit tests: validate URL scopes before giving DNR a rule.
export function isValidDomain(value) {
  return typeof value === 'string' && value.length <= 253 && value.includes('.') && !/^(?:\d{1,3}\.){3}\d{1,3}$/.test(value) &&
    value.split('.').every(s => s.length > 0 && s.length <= 63 && !s.startsWith('-') && !s.endsWith('-') && /^[a-z0-9-]+$/.test(s));
}
export function safePrefix(value) {
  return value == null || (typeof value === 'string' && value.length < 160 && /^\/[a-zA-Z0-9/_~.%-]*$/.test(value) && !value.includes('..'));
}
export function matchesTarget(urlString, rule) {
  try {
    if (!isValidDomain(rule.domain) || !safePrefix(rule.path_prefix)) return false;
    const url = new URL(urlString);
    if (!['http:', 'https:'].includes(url.protocol)) return false;
    const host = url.hostname.toLowerCase();
    if (host !== rule.domain && !host.endsWith('.' + rule.domain)) return false;
    return !rule.path_prefix || url.pathname.startsWith(rule.path_prefix);
  } catch { return false; }
}
export function regexFor(rule) {
  if (!isValidDomain(rule.domain) || !safePrefix(rule.path_prefix)) throw new Error('Invalid block target');
  const domain = rule.domain.replace(/\./g, '\\.');
  const path = rule.path_prefix ? rule.path_prefix.replace(/[.*+?^${}()|[\]\\]/g, '\\$&') : '';
  return `^https?://(?:[^./]+\\.)*${domain}(?::[0-9]+)?${path || '(?:[/?#]|$)'}`;
}

export function allowedVideo(urlString,rule){
 try{const url=new URL(urlString);return url.protocol==='https:'&&url.hostname==='www.youtube.com'&&url.pathname==='/watch'&&Array.isArray(rule.allow_video_ids)&&rule.allow_video_ids.includes(url.searchParams.get('v'));}catch{return false;}
}
export function matchesRule(urlString,rule){
 return matchesTarget(urlString,rule)&&!((rule.action||'block')==='block'&&allowedVideo(urlString,rule));
}
export function learningAllowRules(blocks,startId){
 const hard=blocks.filter(b=>(b.action||'block')==='block');
 const candidates=[...new Set(hard.flatMap(b=>Array.isArray(b.allow_video_ids)?b.allow_video_ids:[]))].filter(id=>typeof id==='string'&&/^[a-zA-Z0-9_-]{5,128}$/.test(id)).slice(0,50);
 return candidates.filter(id=>{const url='https://www.youtube.com/watch?v='+id;const matches=hard.filter(b=>matchesTarget(url,b));return matches.length>0&&matches.every(b=>allowedVideo(url,b));}).map((id,index)=>({id:startId+index,priority:1000,action:{type:'allow'},condition:{regexFilter:'^https://www\\.youtube\\.com/watch\\?v='+id+'(?:&|#|$)',resourceTypes:['main_frame']}}));
}
