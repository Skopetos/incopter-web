import { defineConfig } from 'astro/config';
import tailwind from '@astrojs/tailwind';
export default defineConfig({
  site: 'https://incopter.gr',
  integrations: [tailwind()],
  output: 'static',
  devToolbar: { enabled: false },
});
