import { useEffect, useState } from "react";
import QRCode from "qrcode";
import { Check, Copy, QrCode } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";

interface RemoteDeckQrDialogProps {
  url: string;
  buttonVariant?: "default" | "outline" | "secondary" | "ghost";
  buttonSize?: "default" | "sm" | "lg" | "icon";
  buttonLabel?: string;
}

export function RemoteDeckQrDialog({
  url,
  buttonVariant = "outline",
  buttonSize = "sm",
  buttonLabel = "QR Code",
}: RemoteDeckQrDialogProps) {
  const [open, setOpen] = useState(false);
  const [qrDataUrl, setQrDataUrl] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (!open) {
      return;
    }

    let cancelled = false;
    setError(null);
    setQrDataUrl(null);

    QRCode.toDataURL(url, {
      width: 280,
      margin: 2,
      errorCorrectionLevel: "M",
    })
      .then((dataUrl) => {
        if (!cancelled) {
          setQrDataUrl(dataUrl);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setError(String(err));
        }
      });

    return () => {
      cancelled = true;
    };
  }, [open, url]);

  const handleCopy = async () => {
    await navigator.clipboard.writeText(url);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 2000);
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button variant={buttonVariant} size={buttonSize}>
          <QrCode className="size-3.5" />
          {buttonLabel}
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Quét QR — Remote Deck</DialogTitle>
          <DialogDescription>
            Dùng camera điện thoại quét mã để mở Remote Deck. Thiết bị phải cùng Wi‑Fi với PC.
          </DialogDescription>
        </DialogHeader>

        <div className="flex flex-col items-center gap-4">
          {error && (
            <p className="text-sm text-destructive">Không tạo được QR code: {error}</p>
          )}

          {!error && !qrDataUrl && (
            <div className="flex h-[280px] w-[280px] items-center justify-center rounded-lg border bg-muted text-sm text-muted-foreground">
              Đang tạo mã QR...
            </div>
          )}

          {qrDataUrl && (
            <img
              src={qrDataUrl}
              alt="QR Code Remote Deck"
              className="h-[280px] w-[280px] rounded-lg border bg-white p-2"
            />
          )}

          <code className="w-full break-all rounded-md bg-muted px-3 py-2 text-xs">{url}</code>

          <Button variant="outline" size="sm" onClick={handleCopy} className="w-full">
            {copied ? (
              <>
                <Check className="size-3.5" />
                Đã copy link
              </>
            ) : (
              <>
                <Copy className="size-3.5" />
                Copy link
              </>
            )}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
