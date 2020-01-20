import React, {Component} from 'react';

class Table extends Component {

    state = {};

    feltStyle = {
        fill: "green",
        stroke: "green",
        opacity: 0.5
    };

    render() {
        return (
            <div>
                <svg width="2000" height="600">
                    <rect x="50" y="20" rx="20" ry="20" width="1000" height="500" style={this.feltStyle}/>
                </svg>
            </div>
        );
    };
};

export default Table;