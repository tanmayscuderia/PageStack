import { Component, type ReactNode } from "react";

type Props = {
  children: ReactNode;
};

type State = {
  hasError: boolean;
  message: string;
};

export class ErrorBoundary extends Component<Props, State> {
  state: State = {
    hasError: false,
    message: ""
  };

  static getDerivedStateFromError(error: Error): State {
    return {
      hasError: true,
      message: error.message
    };
  }

  reset = () => {
    this.setState({ hasError: false, message: "" });
  };

  render() {
    if (this.state.hasError) {
      return (
        <div className="appShell">
          <div className="panel notice noticeError" style={{ margin: 20 }}>
            <strong>Something went wrong.</strong>
            <p>{this.state.message}</p>
            <button type="button" className="button secondary" onClick={this.reset}>
              Try again
            </button>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
