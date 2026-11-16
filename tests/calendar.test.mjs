import test from 'node:test';import assert from 'node:assert/strict';
import {parseDay,shiftDay,shiftMonth,monthDays,isoDay} from '../apps/desktop/src/calendar.mjs';
test('calendar validates leap days and rejects rollover dates',()=>{assert.ok(parseDay('2028-02-29'));for(const x of ['2026-02-29','2026-13-01','2026-04-31','05/11/2026',''])assert.equal(parseDay(x),null);});
test('calendar month/year navigation keeps the closest real local day',()=>{assert.equal(shiftMonth('2026-01-31',1),'2026-02-28');assert.equal(shiftMonth('2028-02-29',12),'2029-02-28');assert.equal(shiftDay('2026-12-31',1),'2027-01-01');});
test('calendar grid is six complete Sunday-first weeks with one selected date',()=>{for(const month of ['2026-11-05','2028-02-29','2026-03-01']){const days=monthDays(month);assert.equal(days.length,42);assert.equal(parseDay(days[0]).getDay(),0);assert.equal(new Set(days).size,42);assert.equal(days.filter(x=>x===month).length,1);assert.equal(isoDay(parseDay(month)),month);}});
