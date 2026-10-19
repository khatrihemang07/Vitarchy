import { type StatementPreview, useUploadStatements } from "@/api/finance";
import { Card, CardContent, CardDescription, CardHeader, CardTitle, Notice, Progress } from "@/design";
import { errorMessage } from "../failureMessage";
import { formatCount } from "../money";
import { DropZone } from "./DropZone";

/** The drop zone and the state of the upload in flight. */
export function UploadSection({ onUploaded }: { onUploaded: (previews: StatementPreview[]) => void }) {
  const upload = useUploadStatements();
  return (
    <Card>
      <CardHeader>
        <CardTitle>Import Statements</CardTitle>
        <CardDescription>Each file is read into a Statement you review before it is confirmed.</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <DropZone
          disabled={upload.isPending}
          onFiles={(files) => { upload.mutate(files, { onSuccess: onUploaded }); }}
        />
        {upload.isPending ? (
          <div role="status" className="flex flex-col gap-2 text-sm">
            <p>
              Reading {formatCount(upload.variables.length)} {upload.variables.length === 1 ? "file" : "files"}…
            </p>
            <Progress label="Reading files" />
          </div>
        ) : null}
        {upload.isError ? <Notice kind="failure" inline title={errorMessage(upload.error)} /> : null}
      </CardContent>
    </Card>
  );
}
