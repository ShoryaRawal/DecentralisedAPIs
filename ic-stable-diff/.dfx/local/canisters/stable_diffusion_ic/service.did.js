export const idlFactory = ({ IDL }) => {
  const GenerationRequest = IDL.Record({
    'height' : IDL.Opt(IDL.Nat32),
    'seed' : IDL.Opt(IDL.Nat64),
    'num_inference_steps' : IDL.Opt(IDL.Nat32),
    'prompt' : IDL.Text,
    'width' : IDL.Opt(IDL.Nat32),
    'guidance_scale' : IDL.Opt(IDL.Float32),
    'negative_prompt' : IDL.Opt(IDL.Text),
  });
  const ApiResponse = IDL.Record({
    'data' : IDL.Opt(IDL.Text),
    'error' : IDL.Opt(IDL.Text),
    'timestamp' : IDL.Nat64,
    'success' : IDL.Bool,
  });
  const ApiResponseImage = IDL.Record({
    'data' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'error' : IDL.Opt(IDL.Text),
    'timestamp' : IDL.Nat64,
    'success' : IDL.Bool,
  });
  const TaskStatus = IDL.Variant({
    'Failed' : IDL.Null,
    'Processing' : IDL.Null,
    'Completed' : IDL.Null,
    'Pending' : IDL.Null,
  });
  const GenerationTask = IDL.Record({
    'id' : IDL.Text,
    'status' : TaskStatus,
    'result' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'request' : GenerationRequest,
    'created_at' : IDL.Nat64,
    'error' : IDL.Opt(IDL.Text),
    'completed_at' : IDL.Opt(IDL.Nat64),
  });
  const ApiResponseTask = IDL.Record({
    'data' : IDL.Opt(GenerationTask),
    'error' : IDL.Opt(IDL.Text),
    'timestamp' : IDL.Nat64,
    'success' : IDL.Bool,
  });
  return IDL.Service({
    'generate_image' : IDL.Func([GenerationRequest], [ApiResponse], []),
    'get_image' : IDL.Func([IDL.Text], [ApiResponseImage], ['query']),
    'get_task_status' : IDL.Func([IDL.Text], [ApiResponseTask], ['query']),
    'http_request' : IDL.Func(
        [
          IDL.Record({
            'url' : IDL.Text,
            'method' : IDL.Text,
            'body' : IDL.Vec(IDL.Nat8),
            'headers' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
          }),
        ],
        [
          IDL.Record({
            'body' : IDL.Vec(IDL.Nat8),
            'headers' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
            'streaming_strategy' : IDL.Opt(
              IDL.Variant({
                'Callback' : IDL.Record({
                  'token' : IDL.Opt(IDL.Vec(IDL.Nat8)),
                  'callback' : IDL.Func([], [], ['query']),
                }),
              })
            ),
            'status_code' : IDL.Nat16,
          }),
        ],
        ['query'],
      ),
    'list_tasks' : IDL.Func(
        [],
        [
          IDL.Record({
            'data' : IDL.Opt(IDL.Vec(IDL.Text)),
            'error' : IDL.Opt(IDL.Text),
            'timestamp' : IDL.Nat64,
            'success' : IDL.Bool,
          }),
        ],
        ['query'],
      ),
  });
};
export const init = ({ IDL }) => { return []; };
