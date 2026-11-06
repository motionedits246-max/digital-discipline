// Local calendar dates never pass through UTC parsing (which can shift the day).
export function isoDay(date) { return `${date.getFullYear()}-${String(date.getMonth()+1).padStart(2,'0')}-${String(date.getDate()).padStart(2,'0')}`; }
export function parseDay(value) {
 if(!/^\d{4}-\d{2}-\d{2}$/.test(value))return null;
 const [y,m,d]=value.split('-').map(Number),date=new Date(y,m-1,d,12);
 return y>=1000&&y<=9999&&isoDay(date)===value?date:null;
}
export function shiftDay(value,days) { const date=parseDay(value);if(!date)return value;date.setDate(date.getDate()+days);return isoDay(date); }
export function shiftMonth(value,months) {
 const date=parseDay(value);if(!date)return value;
 const day=date.getDate();date.setDate(1);date.setMonth(date.getMonth()+months);
 const last=new Date(date.getFullYear(),date.getMonth()+1,0,12).getDate();date.setDate(Math.min(day,last));return isoDay(date);
}
export function monthDays(value) {
 const date=parseDay(value);if(!date)return [];
 date.setDate(1);date.setDate(1-date.getDay());
 return Array.from({length:42},(_,i)=>shiftDay(isoDay(date),i));
}
