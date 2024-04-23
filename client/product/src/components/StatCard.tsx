import {
  Card,
  CardContent,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TablePagination,
  TableRow,
  Typography,
} from "@mui/material";
import QueryStatsIcon from "@mui/icons-material/QueryStats";
import "gantt-task-react/dist/index.css";
import { forwardRef, useImperativeHandle, useState } from "react";
import { PieChart } from "@mui/x-charts/PieChart";

interface Column {
  id: "chunk" | "source" | "duration" | "size" | "rate" | "start" | "end";
  label: string;
  minWidth?: number;
  align?: "right";
  format?: (value: number) => string;
}

const columns: readonly Column[] = [
  { id: "chunk", label: "chunk #", minWidth: 170 },
  { id: "source", label: "source", minWidth: 100 },
  {
    id: "duration",
    label: "duration (ms)",
    minWidth: 170,
    align: "right",
    format: (value: number) => value.toLocaleString("en-US"),
  },
  {
    id: "size",
    label: "Size (Bytes)",
    minWidth: 170,
    align: "right",
    format: (value: number) => value.toLocaleString("en-US"),
  },
  {
    id: "rate",
    label: "Rate (Bytes/Sec)",
    minWidth: 170,
    align: "right",
    format: (value: number) => value.toFixed(2),
  },
  {
    id: "start",
    label: "Start",
    minWidth: 170,
    align: "right",
    format: (value: number) => new Date(value).toISOString(),
  },
  {
    id: "end",
    label: "End",
    minWidth: 170,
    align: "right",
    format: (value: number) => new Date(value).toISOString(),
  },
];

interface Data {
  chunk: number;
  source: string;
  duration: number; //milliseconds
  size: number; //KB
  rate: number; //Bytes/sec
  start: number;
  end: number;
}

function createData(
  chunk: number,
  source: string,
  size: number,
  start: number,
  end: number
): Data {
  const duration = end - start;
  const rate = (size / duration) * 1000;
  return { chunk, source, duration, size, rate, start, end };
}

function getPieData(rows: Array<Data>) {
  var nodes = {};
  rows.map((row) => {
    if (nodes[row.source] !== undefined) {
      nodes[row.source].bytes += row.size;
    } else {
      nodes[row.source] = {
        bytes: row.size,
      };
    }
  });
  let ret = [];

  let id = 0;
  for (const [key, val] of Object.entries(nodes)) {
    console.log(key, val);
    ret.push({ key: id, value: val.bytes, label: key });
    id++;
  }
  console.log(ret);

  return ret;
}

const StatCard = forwardRef((props, ref) => {
  const [rows, setRows] = useState<Array<Data>>([]);
  const [page, setPage] = useState(0);
  const [rowsPerPage, setRowsPerPage] = useState(10);

  useImperativeHandle(ref, () => ({
    addStats(data: Data[]) {
      setRows(data);
      // console.log("addStat", data)
    },
  }));

  const handleChangePage = (event: unknown, newPage: number) => {
    setPage(newPage);
  };

  const handleChangeRowsPerPage = (
    event: React.ChangeEvent<HTMLInputElement>
  ) => {
    setRowsPerPage(+event.target.value);
    setPage(0);
  };

  return (
    <>
      <Card sx={{ mt: 5 }}>
        <CardContent>
          <Typography gutterBottom variant="h3" component="div">
            Statistics
            <QueryStatsIcon fontSize="large" sx={{ mx: 2 }} />
          </Typography>
          <PieChart
            series={[
              {
                data: getPieData(rows),
              },
            ]}
            width={600}
            height={200}
            sx={{ my: 2 }}
          />

          <TableContainer>
            <Table stickyHeader aria-label="sticky table">
              <TableHead>
                <TableRow>
                  {columns.map((column) => (
                    <TableCell
                      key={column.id}
                      align={column.align}
                      style={{ minWidth: column.minWidth }}
                    >
                      {column.label}
                    </TableCell>
                  ))}
                </TableRow>
              </TableHead>
              <TableBody>
                {rows
                  .slice(page * rowsPerPage, page * rowsPerPage + rowsPerPage)
                  .map((row) => {
                    return (
                      <TableRow
                        hover
                        role="checkbox"
                        tabIndex={-1}
                        key={row.chunk}
                      >
                        {columns.map((column) => {
                          const value = row[column.id];
                          return (
                            <TableCell key={column.id} align={column.align}>
                              {column.format && typeof value === "number"
                                ? column.format(value)
                                : value}
                            </TableCell>
                          );
                        })}
                      </TableRow>
                    );
                  })}
              </TableBody>
            </Table>
          </TableContainer>
          <TablePagination
            rowsPerPageOptions={[10, 25, 100]}
            component="div"
            count={rows.length}
            rowsPerPage={rowsPerPage}
            page={page}
            onPageChange={handleChangePage}
            onRowsPerPageChange={handleChangeRowsPerPage}
          />
        </CardContent>
      </Card>
    </>
  );
});

export { StatCard, createData };
export type { Data };
