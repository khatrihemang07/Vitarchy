import { api } from "../client";
import { postStatements, uploadForm } from "./upload";

afterEach(() => {
  vi.restoreAllMocks();
});

const pdf = (name: string) => new File(["%PDF"], name, { type: "application/pdf" });

describe("uploadForm", () => {
  it("adds one file part per file under the name the server reads", () => {
    const form = uploadForm([pdf("a.pdf"), pdf("b.pdf")]);
    expect(form.getAll("file").map((part) => (part as File).name)).toEqual(["a.pdf", "b.pdf"]);
  });
});

describe("postStatements", () => {
  it("serializes the files as one multipart form", async () => {
    const post = vi.spyOn(api, "POST");
    let serializer: (() => unknown) | undefined;
    post.mockImplementation(((_path: string, options: { bodySerializer: () => unknown }) => {
      serializer = options.bodySerializer;
      return Promise.resolve({ data: [], response: new Response("[]") });
    }) as never);

    await postStatements([pdf("a.pdf"), pdf("b.pdf")]);

    expect((serializer?.() as FormData).getAll("file")).toHaveLength(2);
  });
});
