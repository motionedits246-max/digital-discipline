import {defineConfig} from '@playwright/test';
export default defineConfig({testDir:'./ui-tests',fullyParallel:false,workers:1,retries:0,timeout:30000,
 reporter:'list',use:{baseURL:'http://127.0.0.1:1420',browserName:'chromium',channel:process.env.PW_CHANNEL||undefined,viewport:{width:1280,height:820},trace:'retain-on-failure'},
 webServer:{command:'npm run dev',url:'http://127.0.0.1:1420',reuseExistingServer:!process.env.CI,timeout:30000},outputDir:'./ui-test-output'});
