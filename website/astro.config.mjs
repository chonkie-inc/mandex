// @ts-check
import { defineConfig } from 'astro/config';
import tailwindcss from '@tailwindcss/vite';
import starlight from '@astrojs/starlight';

export default defineConfig({
  integrations: [
    starlight({
      title: 'mandex',
      logo: {
        light: './public/logo-black.png',
        dark: './public/logo-white.png',
      },
      customCss: ['./src/styles/starlight.css'],
      social: [
        { icon: 'github', label: 'GitHub', href: 'https://github.com/chonkie-inc/mandex' },
      ],
      sidebar: [
        { label: 'Getting Started', link: '/docs/getting-started/' },
        {
          label: 'Commands',
          autogenerate: { directory: 'docs/commands' },
        },
        { label: 'Configuration', link: '/docs/configuration/' },
        {
          label: 'Integrations',
          autogenerate: { directory: 'docs/integrations' },
        },
        { label: 'For Package Authors', link: '/docs/for-authors/' },
      ],
    }),
  ],
  vite: {
    plugins: [tailwindcss()]
  }
});
