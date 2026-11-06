export function createRefreshCoordinator<T>(load:()=>Promise<T>,apply:(value:T)=>void,onError:(error:unknown)=>void):{refresh:(force?:boolean)=>Promise<void>;dispose:()=>void};
