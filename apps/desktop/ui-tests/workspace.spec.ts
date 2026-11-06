import {test,expect} from '@playwright/test';
test('presentation privacy stays visible and restores the existing form draft',async({page})=>{
 await page.goto('/');await page.getByRole('button',{name:'Activity History',exact:true}).click();
 const model=page.getByLabel('Model ID',{exact:true});await model.fill('draft-model-kept');
 await page.getByRole('button',{name:'Presentation mode',exact:true}).click();
 await expect(page.getByRole('heading',{name:'Your activity details are hidden'})).toBeVisible();
 await expect(model).not.toBeVisible();
 await page.getByRole('button',{name:'Show my workspace',exact:true}).click();
 await expect(model).toHaveValue('draft-model-kept');
});
test('custom calendar supports keyboard dates and Escape restores focus',async({page})=>{
 await page.goto('/');await page.getByRole('button',{name:'Activity History',exact:true}).click();
 const trigger=page.getByRole('button',{name:'History date',exact:true});await trigger.click();
 const dialog=page.getByRole('dialog',{name:'History date',exact:true});await expect(dialog).toBeVisible();
 const start=await page.locator('.dd-calendar [data-day]:focus').getAttribute('data-day');
 await page.keyboard.press('ArrowLeft');const moved=await page.locator('.dd-calendar [data-day]:focus').getAttribute('data-day');expect(moved).not.toBe(start);
 await page.keyboard.press('Enter');await expect(dialog).not.toBeVisible();await expect(trigger).toBeFocused();
 await trigger.click();await page.keyboard.press('Escape');await expect(dialog).not.toBeVisible();await expect(trigger).toBeFocused();
});
test('calendar is clamped within a small viewport',async({page})=>{
 await page.setViewportSize({width:560,height:540});await page.goto('/');await page.getByRole('button',{name:'Activity History',exact:true}).click();await page.getByRole('button',{name:'History date',exact:true}).click();
 const box=await page.getByRole('dialog',{name:'History date',exact:true}).boundingBox();expect(box).toBeTruthy();expect(box!.x).toBeGreaterThanOrEqual(0);expect(box!.y).toBeGreaterThanOrEqual(0);expect(box!.x+box!.width).toBeLessThanOrEqual(560);expect(box!.y+box!.height).toBeLessThanOrEqual(540);
});
