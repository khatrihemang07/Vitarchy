import { fireEvent, screen } from "@testing-library/react";
import { jsonResponse, preview, renderWithClient, stubApi } from "../test-utils";
import { UploadSection } from "./UploadSection";

afterEach(() => vi.unstubAllGlobals());

function drop(files: File[]) {
  const input = document.querySelector<HTMLInputElement>('input[type="file"]');
  if (!input) throw new Error("no file input");
  fireEvent.change(input, { target: { files } });
}

const pdf = (name: string) => new File(["x"], name, { type: "application/pdf" });

describe("UploadSection", () => {
  it("shows that files are being read, then hands over the previews", async () => {
    let release: (response: Response) => void = () => undefined;
    stubApi({ "POST /api/finance/statements": () => new Promise<Response>((resolve) => { release = resolve; }) });
    const onUploaded = vi.fn();
    renderWithClient(<UploadSection onUploaded={onUploaded} />);

    drop([pdf("a.pdf"), pdf("b.pdf")]);

    expect(await screen.findByText("Reading 2 files…")).toBeInTheDocument();
    expect(screen.getByRole("progressbar", { name: "Reading files" })).toBeInTheDocument();

    release(jsonResponse(200, [preview({ id: 1 }), preview({ id: 2 })]));
    await vi.waitFor(() => { expect(onUploaded).toHaveBeenCalledTimes(1); });
    expect(screen.queryByText(/Reading 2 files/)).not.toBeInTheDocument();
  });

  it("explains an upload that failed", async () => {
    stubApi({ "POST /api/finance/statements": () => jsonResponse(413, { status: 413, detail: "x", code: "upload_too_large" }) });
    renderWithClient(<UploadSection onUploaded={() => undefined} />);

    drop([pdf("a.pdf")]);

    expect(await screen.findByText(/upload is too large/)).toBeInTheDocument();
  });
});
