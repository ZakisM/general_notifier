import {chromium, type Browser} from 'playwright';

function invariant(condition: any, message: string): asserts condition {
    if (condition) return;

    throw new Error(message);
}

let browser: Browser;

const getPageSource = async (url: string, timeout: number) => {
    if (!browser) {
        try {
            browser = await chromium.launch({headless: true});
        } catch (error) {
            console.error(error);
            process.exit(1);
        }
    }

    let res: string | undefined;

    try {
        const page = await browser.newPage();
        await page.route(
            '**/*.{css,png,jpg,jpeg,mp4,mp3,ttf,ttf2,woff,woff2,webp,svg,xml}',
            (route) => route.abort(),
        );
        await page.goto(url, {timeout: timeout * 1000});

        res = await page.content();

        await page.close();
    } catch (error) {
        console.error(`Failed to get ${URL} due to: ${error}`);
    }

    invariant(
        res !== undefined,
        `Response was undefined when trying to to read page source for: ${URL}`,
    );

    return res;
};

Bun.serve({
    development: false,
    async fetch(req: Request) {
        const {searchParams} = new URL(req.url);

        const url = searchParams.get('url');
        invariant(url !== null, 'url was null');

        const timeout = Number(searchParams.get('timeout'));
        invariant(
            timeout !== null && !Number.isNaN(timeout),
            'timeout was null | NaN',
        );

        try {
            const source = await getPageSource(url, Number(timeout));

            return new Response(source);
        } catch (error) {
            throw new Error(
                `Failed to call 'getPageSource' for '${URL}' due to: ${error}`,
            );
        }
    },
    port: 8050,
});
