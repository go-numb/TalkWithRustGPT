import { Image } from "antd";

// imageがあれば、表示するコンポネント
export const ImageComponent = ({ images, size }: { images: string[], size: number }) => {
    return (
        images.map((image, index) => (
            <Image
                key={index}
                width={size}
                src={image}
            />
        ))
    );
}


/**
 * 画像ファイルをリサイズし、Base64データURLに変換する関数。
 *
 * @param {File} file - 入力画像ファイル
 * @param {number} maxWidth - 最大幅（px）
 * @param {number} maxHeight - 最大高さ（px）
 * @returns {Promise<string>} リサイズ後のBase64データURL
 */
export const resizeImageAndConvertToBase64 = (file: File, maxWidth: number, maxHeight: number): Promise<string> => {
    return new Promise((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = (event) => {
            const img: HTMLImageElement = document.createElement("img");
            img.onload = () => {
                const canvas = document.createElement('canvas');
                const ctx = canvas.getContext('2d');
                if (!ctx) {
                    reject(new Error('Failed to get canvas context'));
                    return;
                }
                const originalWidth = img.width;
                const originalHeight = img.height;
                let newWidth = originalWidth;
                let newHeight = originalHeight;
                if (originalWidth > maxWidth || originalHeight > maxHeight) {
                    const widthRatio = maxWidth / originalWidth;
                    const heightRatio = maxHeight / originalHeight;
                    const bestRatio = Math.min(widthRatio, heightRatio);
                    newWidth = originalWidth * bestRatio;
                    newHeight = originalHeight * bestRatio;
                }
                canvas.width = newWidth;
                canvas.height = newHeight;
                ctx.drawImage(img, 0, 0, newWidth, newHeight);
                resolve(canvas.toDataURL('image/png'));
            };
            img.src = event.target!.result as string;
        };
        reader.onerror = (error) => {
            reject(new Error('Failed to read file: ' + error));
        };
        reader.readAsDataURL(file);
    });
};