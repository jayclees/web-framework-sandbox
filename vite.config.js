import { defineConfig } from 'vite'

export default defineConfig({
    logLevel: 'info',
    server: {
        cors: {
            // the origin you will be accessing via browser
            origin: 'http://172.19.0.2',
        },
        origin: 'http://172.19.0.2',
        watch: {

        },
    },
    build: {
        // generate .vite/manifest.json in outDir
        manifest: true,
        rolldownOptions: {
            input: './resource/js/main.js',
        },
        outDir: './public/dist',
        modulePreload: {
            polyfill: true,
        },
    },
    publicDir: false,
    css: {
        preprocessorOptions: {
            scss: {
                silenceDeprecations: [
                    'import',
                    'mixed-decls',
                    'color-functions',
                    'global-builtin',
                    'if-function',
                ],
            },
        },
    },
    plugins: [
        watchResourceDir(),
    ]
})

function watchResourceDir() {
    return {
        name: 'vite-plugin-sturdy-framework',
        handleHotUpdate(ctx) {
            if (ctx.file.startsWith(`${__dirname}/resource`)) {
                ctx.server.ws.send({ type: 'full-reload' })
            }

            return []
        },
    }
}
