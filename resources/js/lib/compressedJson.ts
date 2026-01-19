export async function fetchCompressedJson<T>(url: string): Promise<T> {
    const response = await fetch(url, { cache: 'force-cache' });

    if (!response.ok) {
        throw new Error(`Failed to fetch compressed JSON: ${response.status}`);
    }

    const contentEncoding = response.headers.get('content-encoding');

    if (contentEncoding === 'gzip') {
        return (await response.json()) as T;
    }

    if (typeof DecompressionStream === 'undefined') {
        throw new Error('DecompressionStream is not supported in this browser.');
    }

    if (!response.body) {
        throw new Error('Response body is unavailable for decompression.');
    }

    const decompressedStream = response.body.pipeThrough(new DecompressionStream('gzip'));
    const decompressedResponse = new Response(decompressedStream);

    return (await decompressedResponse.json()) as T;
}
